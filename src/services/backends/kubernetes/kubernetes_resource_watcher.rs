use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use async_trait::async_trait;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::watcher;
use kube::Resource;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[async_trait]
pub trait ResourceUpdateHandler<S>: Send + Sync
where
    S: Resource + Send + Sync,
{
    async fn handle_update(&self, result: Result<S, watcher::Error>) -> ();
}

#[async_trait]
pub trait KubernetesResourceWatcher<R>: Sized
where
    R: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<R> + Send + Sync + 'static;

    fn stop(&self) -> anyhow::Result<()>;
}
