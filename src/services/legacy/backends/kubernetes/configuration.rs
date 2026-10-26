use crate::services::audit::audit_facade::WithAuditFacade;
use crate::services::audit::log_audit_service::LogAuditService;
use crate::services::backends::BackendConfiguration;
use crate::services::backends::kubernetes::kubeconfig_loader::{from_cluster, from_command, from_file};
use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use crate::services::backends::kubernetes::kubernetes_resource_manager::{
    GenericKubernetesResourceManager, KubernetesResourceManagerConfig, UpdateLabels,
};
use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, KubernetesResourceWatcherRunner,
};
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::services::legacy::backends::BackendBuilder;
use crate::services::legacy::backends::kubernetes::KubernetesBackend;
use crate::services::legacy::configuration::models::KubernetesBackendSettings;
use crate::services::legacy::prefix_tree::parametrized_matcher::ParametrizedMatcher;
use crate::services::legacy::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::legacy::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::legacy::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::legacy::repositories::lookup_trie::{EntityCollectionResource, SchemaBoundResource};
use crate::services::legacy::repositories::policy_repository;
use crate::services::legacy::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::legacy::repositories::policy_repository::read_only::PolicyRepositoryData;
use crate::services::legacy::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::observability::open_telemetry::tracing::tracing_facade::WithTracingFacade;
use crate::services::validation_service::path_segment::PathSegment;
use anyhow::bail;
use async_trait::async_trait;
use cedar_policy::{EntityUid, PolicySet};
use k8s_openapi::NamespaceResourceScope;
use kube::Config;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio;

#[async_trait]
impl BackendConfiguration for BackendBuilder {
    type BackendSettings = KubernetesBackendSettings;
    type InitializedBackend = KubernetesBackend;

    async fn configure(
        self,
        settings: &Self::BackendSettings,
        instance_name: String,
    ) -> anyhow::Result<Arc<Self::InitializedBackend>> {
        let kubeconfig = Self::get_kubeconfig(settings).await?;
        let owner_mark = ObjectOwnerMark::new(&settings.resource_owner_label, &instance_name);

        let (schema_repository, schema_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let schema_repository = schema_repository.with_audit(Arc::new(LogAuditService::new()));

        let action_lookup_table_listener = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
            "action_lookup".to_string(),
        )
        .await?;

        let (action_repository, action_repository_readiness): (_, tokio::sync::oneshot::Receiver<()>) =
            Self::create_repository(
                &settings.namespace,
                kubeconfig.clone(),
                owner_mark.clone(),
                settings.operation_timeout.into(),
            )
            .await?;
        let action_repository: Arc<ActionDataRepository> =
            action_repository.with_audit(Arc::new(LogAuditService::new()));

        let resource_lookup_table_listener = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
            "resource_lookup".to_string(),
        )
        .await?;

        let (resource_repository, resource_repository_readiness): (_, tokio::sync::oneshot::Receiver<()>) =
            Self::create_repository(
                &settings.namespace,
                kubeconfig.clone(),
                owner_mark.clone(),
                settings.operation_timeout.into(),
            )
            .await?;
        let resource_repository: Arc<ResourceDiscoveryDocumentRepository> =
            resource_repository.with_audit(Arc::new(LogAuditService::new()));

        let policy_lookup_watcher = Self::create_readonly_repository::<PathSegment>(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;

        let (policy_repository, policy_repository_readiness) = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.operation_timeout.into(),
        )
        .await?;
        let policy_repository = policy_repository.with_audit(Arc::new(LogAuditService::new()));

        let readiness_state = Arc::new(AtomicBool::new(false));
        let readiness_state_update = readiness_state.clone();
        tokio::spawn(async move {
            let is_ready = schema_repository_readiness.await.is_ok()
                && action_repository_readiness.await.is_ok()
                && resource_repository_readiness.await.is_ok()
                && policy_repository_readiness.await.is_ok();
            readiness_state_update.store(is_ready, Ordering::Release);
        });

        Ok(Arc::new(KubernetesBackend {
            schema_repository,
            action_repository,
            resource_repository,
            policy_repository,
            readiness_state,
            action_lookup_table_listener,
            resource_lookup_table_listener,
            policy_lookup_watcher,
        }))
    }
}

impl BackendBuilder {
    async fn get_kubeconfig(settings: &KubernetesBackendSettings) -> anyhow::Result<Config> {
        match settings {
            KubernetesBackendSettings { in_cluster: true, .. } => from_cluster().load(),

            KubernetesBackendSettings {
                exec: Some(command), ..
            } => from_command().load(command).await,

            KubernetesBackendSettings {
                kubeconfig: Some(path), ..
            } => from_file().load(path).await,

            KubernetesBackendSettings {
                kubeconfig: None,
                exec: None,
                ..
            } => {
                bail!("Kubernetes backend configuration is missing")
            }
        }
    }

    pub async fn create_repository<R>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<(
        Arc<KubernetesRepository<R, GenericKubernetesResourceManager<R>>>,
        tokio::sync::oneshot::Receiver<()>,
    )>
    where
        R: kube::Resource<Scope = NamespaceResourceScope>
            + SoftDeleteResource
            + UpdateLabels
            + Clone
            + Send
            + Sync
            + 'static,
        R::DynamicType: Hash + Eq + Clone + Default,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        let (resource_manager, readiness_rx) =
            GenericKubernetesResourceManager::start(config, Arc::new(LoggingUpdateHandler)).await?;
        let repository =
            KubernetesRepository::<R, GenericKubernetesResourceManager<R>>::start(resource_manager, operation_timeout)
                .await
                .map(Arc::new)?;
        Ok((repository, readiness_rx))
    }

    pub async fn create_lookup_trie<R, K>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
        operation_name: String,
    ) -> anyhow::Result<
        Arc<ReadOnlyRepositoryBackend<SchemaBoundedTrieRepositoryData<K>, R, (String, Vec<K>), EntityUid>>,
    >
    where
        K: Debug + Ord + Clone + Send + Sync + Hash + 'static + ParametrizedMatcher,
        R: kube::Resource<Scope = NamespaceResourceScope>
            + SoftDeleteResource
            + EntityCollectionResource<K>
            + UpdateLabels
            + Clone
            + SchemaBoundResource
            + Send
            + Sync
            + 'static,
        R::DynamicType: Hash + Eq + Clone + Default,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        let lookup_trie = Arc::new(SchemaBoundedTrieRepositoryData::<K>::new());
        let mut r = ReadOnlyRepositoryBackend::new(lookup_trie.clone(), lookup_trie.with_tracing(operation_name));
        r.start(config).await?;
        Ok(Arc::new(r))
    }

    pub async fn create_readonly_repository<K>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<Arc<ReadOnlyRepositoryBackend<PolicyRepositoryData, PolicyDocument, String, PolicySet>>>
    where
        K: Ord,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        let lookup_trie = policy_repository::read_only::new();
        let mut r = ReadOnlyRepositoryBackend::new(
            lookup_trie.clone(),
            lookup_trie.with_tracing("policy_lookup".to_string()),
        );
        r.start(config).await?;
        Ok(Arc::new(r))
    }
}
