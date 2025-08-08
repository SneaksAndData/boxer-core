use crate::services::backends::kubernetes::kubernetes_resource_manager::versioned::VersionedKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::services::backends::kubernetes::repositories::schema_repository::models::SchemaDocument;
use crate::testing::api_client_context::ApiClientContext;
use std::sync::Arc;
use test_context::AsyncTestContext;

pub struct VersionedKubernetesResourceManagerTestContext {
    pub manager: VersionedKubernetesResourceManager<SchemaDocument>,
    pub config: KubernetesResourceManagerConfig,
    pub api_context: ApiClientContext<SchemaDocument>,
}

impl VersionedKubernetesResourceManagerTestContext {}

impl AsyncTestContext for VersionedKubernetesResourceManagerTestContext {
    async fn setup() -> Self {
        let api_context: ApiClientContext<SchemaDocument> = ApiClientContext::setup().await;

        let config = KubernetesResourceManagerConfig {
            namespace: api_context.namespace().to_string(),
            label_selector_key: "repository.boxer.io/type".to_string(),
            label_selector_value: "versioned-resource".to_string(),
            kubeconfig: api_context.config().clone(),
            field_manager: "boxer".to_string(),
        };

        let manager = VersionedKubernetesResourceManager::start(config.clone(), Arc::new(LoggingUpdateHandler))
            .await
            .expect("Failed to start VersionedKubernetesResourceManager");

        VersionedKubernetesResourceManagerTestContext {
            manager,
            config,
            api_context,
        }
    }
}
