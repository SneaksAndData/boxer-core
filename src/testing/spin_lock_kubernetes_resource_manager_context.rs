use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::kubernetes_resource_manager::spin_lock::SpinLockKubernetesResourceManager;
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::services::backends::kubernetes::repositories::schema_repository::models::SchemaDocument;
use crate::testing::api_client_context::ApiClientContext;
use std::sync::Arc;
use test_context::AsyncTestContext;

pub struct SpinLockKubernetesResourceManagerTestContext {
    pub manager: SpinLockKubernetesResourceManager<SchemaDocument>,
    pub config: KubernetesResourceManagerConfig,
    pub api_context: ApiClientContext<SchemaDocument>,
}

impl SpinLockKubernetesResourceManagerTestContext {}

impl AsyncTestContext for SpinLockKubernetesResourceManagerTestContext {
    async fn setup() -> Self {
        let api_context: ApiClientContext<SchemaDocument> = ApiClientContext::setup().await;

        let config = KubernetesResourceManagerConfig {
            namespace: api_context.namespace().to_string(),
            label_selector_key: "repository.boxer.io/type".to_string(),
            label_selector_value: "resource".to_string(),
            kubeconfig: api_context.config().clone(),
            field_manager: "boxer".to_string(),
            operation_timeout: std::time::Duration::from_secs(30),
        };

        let manager = SpinLockKubernetesResourceManager::start(config.clone(), Arc::new(LoggingUpdateHandler))
            .await
            .expect("Failed to start SpinLockKubernetesResourceManager");

        SpinLockKubernetesResourceManagerTestContext {
            manager,
            config,
            api_context,
        }
    }
}
