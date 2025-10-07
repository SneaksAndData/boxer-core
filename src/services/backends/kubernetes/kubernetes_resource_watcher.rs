use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::resource_update_handler::ResourceUpdateHandler;
use async_trait::async_trait;
use k8s_openapi::NamespaceResourceScope;
use kube::Resource;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[async_trait]
pub trait KubernetesResourceWatcher<H, R>: Sized
where
    R: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<R> + Send + Sync + 'static,
{
    async fn start(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>;

    fn stop(&self) -> anyhow::Result<()>;
}

#[async_trait]
pub trait KubernetesResourceWatcherRunner<H, R>: Sized
where
    R: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<R> + Send + Sync + 'static,
{
    async fn start(&mut self, config: KubernetesResourceManagerConfig) -> anyhow::Result<()>;

    fn stop(&self) -> anyhow::Result<()>;
}
