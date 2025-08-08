pub mod status;
pub mod versioned;

use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use anyhow::{Error, anyhow};
use async_trait::async_trait;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams, PostParams};
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::watcher::Config;
use kube::runtime::{WatchStreamExt, reflector, watcher};
use kube::{Api, Client, Resource};
use log::debug;
use maplit::btreemap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

pub trait UpdateLabels: Resource<Scope = NamespaceResourceScope> {
    fn update_labels(self, custom_labels: &mut BTreeMap<String, String>) -> Self;
}

/// Configuration for the Kubernetes repository.
#[derive(Clone)]
pub struct KubernetesResourceManagerConfig {
    pub namespace: String,
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub kubeconfig: kube::Config,

    field_manager: String,
}

impl KubernetesResourceManagerConfig {
    #[allow(dead_code)]
    pub fn clone_with_label_selector(&self, label_selector_key: String, label_selector_value: String) -> Self {
        KubernetesResourceManagerConfig {
            namespace: self.namespace.clone(),
            label_selector_key,
            label_selector_value,
            kubeconfig: self.kubeconfig.clone(),
            field_manager: self.field_manager.clone(),
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

    #[allow(dead_code)]
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
        self.api
            .patch(&object_ref.name, &PatchParams::apply("boxer"), &patch)
            .await
    }

    pub async fn update(&self, name: String, new_object: &S) -> Result<S, Error> {
        self.api
            .replace(&name, &PostParams::default(), new_object)
            .await // TODO: add field manager
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
            .create(&PostParams::default(), &new_object)
            .await // TODO: add field manager
            .map_err(|e| anyhow!("Failed to create resource: {}", e))
    }

    pub fn get(&self, object_ref: &ObjectRef<S>) -> Option<Arc<S>> {
        self.reader.get(object_ref)
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
        let watcher_config = Config {
            label_selector: Some(format!("{}={}", config.label_selector_key, config.label_selector_value)),
            ..Default::default()
        };
        let stream = watcher(api.clone(), watcher_config);
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

        let custom_labels = btreemap! {
            config.label_selector_key.clone() => config.label_selector_value.clone(),
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
