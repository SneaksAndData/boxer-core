pub mod spin_lock;
pub mod status;

use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams, PostParams};
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::watcher::Config;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
use maplit::btreemap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

pub trait UpdateLabels: Resource<Scope = NamespaceResourceScope> {
    fn update_labels(self, custom_labels: &mut BTreeMap<String, String>) -> Self;
}

/// Configuration for the Kubernetes repository.
#[derive(Debug, Clone)]
pub struct KubernetesResourceManagerConfig {
    pub namespace: String,
    pub kubeconfig: kube::Config,
    pub field_manager: String,
    pub listener_config: ListenerConfig,
}

#[derive(Debug, Clone)]
pub struct ListenerConfig {
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub operation_timeout: Duration,
}

impl Into<Config> for &ListenerConfig {
    fn into(self) -> Config {
        Config {
            label_selector: Some(format!("{}={}", self.label_selector_key, self.label_selector_value)),
            ..Default::default()
        }
    }
}

pub struct KubernetesResourceManager<StoredObject>
where
    StoredObject: Resource + 'static,
    StoredObject::DynamicType: Hash + Eq,
{
    reader: Store<StoredObject>,
    handle: tokio::task::JoinHandle<()>,
    api: Api<StoredObject>,
    namespace: String,
    field_manager: String,
    pub custom_labels: BTreeMap<String, String>,
}

impl<S> KubernetesResourceManager<S>
where
    S: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(
        reader: Store<S>,
        handle: tokio::task::JoinHandle<()>,
        api: Api<S>,
        namespace: String,
        field_manager: String,
        custom_labels: BTreeMap<String, String>,
    ) -> Self {
        KubernetesResourceManager {
            reader,
            handle,
            api,
            namespace,
            field_manager,
            custom_labels,
        }
    }

    pub fn namespace(&self) -> String {
        self.namespace.clone()
    }

    pub async fn replace(&self, _: &str, object: S) -> Result<(), Error> {
        let object_name = object
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Object name is required for replacement"))?;

        let exists = self
            .api
            .get(&object_name)
            .await
            .map(|_| true)
            .or_else(|_| Ok::<bool, Error>(false))?;

        if exists {
            debug!("Replacing existing resource: {}", object_name);
            self.api
                .replace(&object_name, &PostParams::default(), &object)
                .await
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to update resource: {}", e))
        } else {
            debug!("Creating new resource: {}", object_name);
            self.api
                .create(&PostParams::default(), &object)
                .await
                .map(|_| ())
                .map_err(|e| anyhow!("Failed to create resource: {}", e))
        }
    }

    pub async fn replace_object(&self, object_ref: ObjectRef<S>, new_object: S) -> Result<S, Error> {
        let old_object = self.get(&object_ref);
        match old_object {
            None => self.create(new_object).await,
            Some(_) => self.update(object_ref.name, &new_object).await,
        }
    }

    pub async fn upsert_object(&self, object_ref: &ObjectRef<S>, resource: S) -> Result<S, kube::Error> {
        let patch = Patch::Apply(resource.update_labels(&mut self.custom_labels.clone()));
        self.api.patch(&object_ref.name, &self.patch_params(), &patch).await
    }

    pub async fn update(&self, name: String, new_object: &S) -> Result<S, Error> {
        self.api
            .replace(&name, &self.post_params(), new_object)
            .await
            .map_err(|e| anyhow!("Failed to create resource: {}", e))
    }

    pub async fn create(&self, mut new_object: S) -> Result<S, Error> {
        debug!(
            "Creating new resource: {}",
            new_object.meta().name.as_deref().unwrap_or("unknown")
        );
        let mut labels = new_object.meta().clone().labels.unwrap_or_default();
        labels.extend(self.custom_labels.clone());
        new_object.meta_mut().labels = Some(labels);
        debug!("Labels: {:?}", new_object.meta().labels);

        self.api
            .create(&self.post_params(), &new_object)
            .await
            .map_err(|e| anyhow!("Failed to create resource: {}", e))
    }

    pub fn get(&self, object_ref: &ObjectRef<S>) -> Option<Arc<S>> {
        self.reader.get(object_ref)
    }

    fn post_params(&self) -> PostParams {
        PostParams {
            field_manager: Some(self.field_manager.clone()),
            ..Default::default()
        }
    }

    fn patch_params(&self) -> PatchParams {
        PatchParams {
            field_manager: Some(self.field_manager.clone()),
            ..Default::default()
        }
    }
}

#[async_trait]
impl<S> KubernetesResourceWatcher<S> for KubernetesResourceManager<S>
where
    S: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let stream = watcher(api.clone(), (&config.listener_config).into());
        let (reader, writer) = reflector::store();

        let reflector = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(move |r| {
                let update_handler = update_handler.clone();
                async move {
                    update_handler.handle_update(r).await;
                }
            });

        let handle = tokio::spawn(reflector);
        reader.wait_until_ready().await?;

        let label_selector_key = config.listener_config.label_selector_key.clone();
        let label_selector_value = config.listener_config.label_selector_value.clone();
        let custom_labels = btreemap! {
            label_selector_key => label_selector_value,
        };
        Ok(KubernetesResourceManager::new(
            reader,
            handle,
            api,
            config.namespace,
            config.field_manager,
            custom_labels,
        ))
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}
