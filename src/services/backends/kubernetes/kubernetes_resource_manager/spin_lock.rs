mod tests;

use super::{KubernetesResourceManager, KubernetesResourceManagerConfig, ResourceUpdateHandler, UpdateLabels};
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use anyhow::Error;
use kube::runtime::reflector::ObjectRef;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub struct SpinLockKubernetesResourceManager<R>
where
    R: kube::Resource + 'static,
    R::DynamicType: Hash + Eq,
{
    resource_manager: KubernetesResourceManager<R>,
}

impl<R> SpinLockKubernetesResourceManager<R>
where
    R: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(resource_manager: KubernetesResourceManager<R>) -> Self {
        SpinLockKubernetesResourceManager { resource_manager }
    }

    pub async fn upsert(&self, object_ref: &ObjectRef<R>, resource: R) -> Result<R, Status> {
        self.resource_manager
            .upsert_object(object_ref, resource)
            .await
            .map_err(Status::from)
    }

    pub fn get(&self, object_ref: &ObjectRef<R>) -> Result<Arc<R>, Status> {
        let result = self.resource_manager.get(&object_ref);
        match result {
            None => Err(Status::NotFound(object_ref.into())),
            Some(resource) => Ok(resource),
        }
    }

    pub async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> Result<Self, Error>
    where
        H: ResourceUpdateHandler<R> + Send + Sync + 'static,
    {
        let resource_manager = KubernetesResourceManager::start(config, update_handler).await?;
        Ok(SpinLockKubernetesResourceManager::new(resource_manager))
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        <KubernetesResourceManager<R> as KubernetesResourceWatcher<LoggingUpdateHandler, R>>::stop(
            &self.resource_manager,
        )
    }

    pub fn namespace(&self) -> String {
        self.resource_manager.namespace.clone()
    }
}
