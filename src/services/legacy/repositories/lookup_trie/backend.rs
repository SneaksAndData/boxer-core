use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcherRunner, ResourceUpdateHandler,
};
use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::service_provider::ServiceProvider;
use async_trait::async_trait;
use futures::stream::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::{WatchStreamExt, reflector, watcher};
use kube::{Api, Client, Resource};
use log::debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub type AssociatedRepository<Key, Value> = dyn ReadOnlyRepository<Key, Value, ReadError = anyhow::Error>;

pub struct ReadOnlyRepositoryBackend<H, S, K, V>
where
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    handle: Option<JoinHandle<()>>,
    update_handler: Arc<H>,
    repository: Arc<AssociatedRepository<K, V>>,
    _marker: std::marker::PhantomData<S>,
}

impl<H, S, K, V> ReadOnlyRepositoryBackend<H, S, K, V>
where
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(update_handler: Arc<H>, repository: Arc<AssociatedRepository<K, V>>) -> Self {
        ReadOnlyRepositoryBackend {
            handle: None,
            update_handler,
            _marker: std::marker::PhantomData,
            repository,
        }
    }
}

#[async_trait]
impl<H, S, K, V> KubernetesResourceWatcherRunner<H, S> for ReadOnlyRepositoryBackend<H, S, K, V>
where
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    S::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
{
    async fn start(&mut self, config: KubernetesResourceManagerConfig) -> anyhow::Result<()> {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let watcher_config = (&config.owner_mark).into();
        let stream = watcher(api, watcher_config);
        let (reader, writer) = reflector::store();

        let handler = self.update_handler.clone();
        let reflector = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(move |r| {
                let update_handler = handler.clone();
                async move {
                    update_handler.handle_update(r).await;
                }
            });

        let handle = tokio::spawn(reflector);
        reader.wait_until_ready().await?;
        self.handle = Some(handle);
        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}

impl<H, S, K, V> ServiceProvider<Arc<AssociatedRepository<K, V>>> for ReadOnlyRepositoryBackend<H, S, K, V>
where
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
    K: Ord + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    fn get(&self) -> Arc<AssociatedRepository<K, V>> {
        self.repository.clone()
    }
}
