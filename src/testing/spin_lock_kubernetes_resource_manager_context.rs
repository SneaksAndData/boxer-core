use crate::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use crate::services::backends::kubernetes::kubernetes_resource_manager::{
    GenericKubernetesResourceManager, KubernetesResourceManagerConfig, UpdateLabels,
};
use crate::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::testing::api_client_context::ApiClientContext;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use test_context::AsyncTestContext;

pub struct GenericKubernetesResourceManagerTestContext<R>
where
    R: kube::Resource + 'static,
    R::DynamicType: Hash + Eq,
{
    pub manager: GenericKubernetesResourceManager<R>,
    pub config: KubernetesResourceManagerConfig,
    pub api_context: ApiClientContext<R>,
}

impl<R> AsyncTestContext for GenericKubernetesResourceManagerTestContext<R>
where
    R: kube::Resource + UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    async fn setup() -> Self {
        let api_context: ApiClientContext<R> = ApiClientContext::setup().await;

        let config = KubernetesResourceManagerConfig {
            namespace: api_context.namespace().to_string(),
            kubeconfig: api_context.config().clone(),
            owner_mark: ObjectOwnerMark::new("boxer.io", "unit-tests"),
            operation_timeout: Duration::from_secs(10),
        };

        let manager = GenericKubernetesResourceManager::start(config.clone(), Arc::new(LoggingUpdateHandler))
            .await
            .expect("Failed to start SpinLockKubernetesResourceManager");

        GenericKubernetesResourceManagerTestContext {
            manager,
            config,
            api_context,
        }
    }
}
