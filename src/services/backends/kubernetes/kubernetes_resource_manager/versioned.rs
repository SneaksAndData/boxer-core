mod tests;

use super::{KubernetesResourceManager, KubernetesResourceManagerConfig, ResourceUpdateHandler, UpdateLabels};
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use anyhow::Error;
use kube::runtime::reflector::ObjectRef;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub struct VersionedKubernetesResourceManager<R>
where
    R: kube::Resource + 'static,
    R::DynamicType: Hash + Eq,
{
    resource_manager: KubernetesResourceManager<R>,
}

impl<R> VersionedKubernetesResourceManager<R>
where
    R: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(resource_manager: KubernetesResourceManager<R>) -> Self {
        VersionedKubernetesResourceManager { resource_manager }
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
        let resource_manager = KubernetesResourceManager::start(config.clone(), update_handler).await?;
        Ok(VersionedKubernetesResourceManager::new(resource_manager))
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.resource_manager.stop()
    }

    pub fn namespace(&self) -> String {
        self.resource_manager.namespace.clone()
    }
}
