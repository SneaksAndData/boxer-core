use super::*;
use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcherRunner;
use crate::services::legacy::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::legacy::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::legacy::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use crate::services::legacy::repositories::resource_repository::resource_route_registration::{
    ResourceRouteRegistration, ResourceSetRegistration,
};
use crate::services::service_provider::ServiceProvider;
use crate::services::validation_service::request_context::RequestContext;
use crate::testing::api_extensions::WaitForResource;
use crate::testing::spin_lock_kubernetes_resource_manager_context::GenericKubernetesResourceManagerTestContext;
use cedar_policy::EntityUid;
use kube::Api;
use std::sync::Arc;
use std::time::Duration;
use test_context::{AsyncTestContext, test_context};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesResourceRepositoryTest {
    repository: Arc<ResourceDiscoveryDocumentRepository>,
    api: Api<ResourceDiscoveryDocument>,
    namespace: String,
    lookup: ReadOnlyRepositoryBackend<
        SchemaBoundedTrieRepositoryData<PathSegment>,
        ResourceDiscoveryDocument,
        (String, Vec<PathSegment>),
        EntityUid,
    >,
}

impl AsyncTestContext for KubernetesResourceRepositoryTest {
    async fn setup() -> Self {
        let parent = GenericKubernetesResourceManagerTestContext::setup().await;
        let owner_mark = parent.config.owner_mark.clone();
        let operation_timeout = parent.config.operation_timeout.clone();
        let config = KubernetesResourceManagerConfig {
            namespace: parent.config.namespace.clone(),
            kubeconfig: parent.config.kubeconfig.clone(),
            owner_mark,
            operation_timeout: operation_timeout.clone(),
        };
        let lookup_trie = Arc::new(SchemaBoundedTrieRepositoryData::<PathSegment>::new());
        let mut lookup = ReadOnlyRepositoryBackend::new(lookup_trie.clone(), lookup_trie.clone());
        lookup.start(config).await.unwrap();

        let repository = Arc::new(
            KubernetesRepository::start(parent.manager, parent.config.operation_timeout.clone())
                .await
                .unwrap(),
        );
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            lookup,
        }
    }
}

#[test_context(KubernetesResourceRepositoryTest)]
#[tokio::test]
async fn test_create_resource_discovery_document(ctx: &mut KubernetesResourceRepositoryTest) {
    // Arrange
    let name = "resource-discovery-document";
    let registration = ResourceSetRegistration {
        hostname: "www.example.com".to_string(),
        routes: vec![ResourceRouteRegistration {
            route_template: "api/v1/{object}".to_string(),
            resource_uid: "PhotoApp::Photo::\"vacationPhoto.jpg\"".to_string(),
        }],
    };

    ctx.repository
        .upsert(
            ("schema".to_string(), name.to_string()),
            registration.with_schema("schema".to_string()),
        )
        .await
        .expect("Failed to upsert schema");

    // Act
    let key = format!("{}-{}", "schema", name);
    ctx.api
        .wait_for_creation(key, ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let after = ctx.lookup.get().get(("schema".to_string(), key)).await;

    // Assert
    assert!(after.is_ok());
}
