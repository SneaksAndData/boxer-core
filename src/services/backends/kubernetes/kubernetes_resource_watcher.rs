use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use async_trait::async_trait;
use futures::future::Ready;
use k8s_openapi::NamespaceResourceScope;
use kube::Resource;
use kube::runtime::watcher;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub trait ResourceUpdateHandler<S>: Send + Sync
where
    S: Resource + Send + Sync,
{
    fn handle_update(&self, result: Result<S, watcher::Error>) -> impl Future<Output = ()> + Send + Sync ;
}

#[async_trait]
pub trait KubernetesResourceWatcher<R, H>: Sized
where
    R: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<R> + Send + Sync + 'static,
{
    async fn start(
        config: KubernetesResourceManagerConfig,
        update_handler: Arc<H>,
    ) -> anyhow::Result<Self>;
    
    fn stop(&self) -> anyhow::Result<()>;
}
