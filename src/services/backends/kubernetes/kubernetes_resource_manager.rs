pub mod object_owner_mark;
pub mod spin_lock;
pub mod status;

use crate::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::owner_conflict_details::OwnerConflictDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status::{Conflict, NotOwned};
use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams, PostParams};
use kube::core::ErrorResponse;
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
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
    pub owner_mark: ObjectOwnerMark,
    pub operation_timeout: Duration,
}

pub struct KubernetesResourceManager<R>
where
    R: Resource + 'static,
    R::DynamicType: Hash + Eq,
{
    reader: Store<R>,
    handle: tokio::task::JoinHandle<()>,
    api: Api<R>,
    namespace: String,
    owner_mark: ObjectOwnerMark,
}

impl<S> KubernetesResourceManager<S>
where
    S: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    fn new(
        reader: Store<S>,
        handle: tokio::task::JoinHandle<()>,
        api: Api<S>,
        namespace: String,
        owner_mark: ObjectOwnerMark,
    ) -> Self {
        KubernetesResourceManager {
            reader,
            handle,
            api,
            namespace,
            owner_mark,
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

    pub async fn upsert_object(&self, object_ref: &ObjectRef<S>, resource: S) -> Result<S, Status> {
        let mut owner_labels: BTreeMap<String, String> = (&self.owner_mark).into();
        let patch = Patch::Apply(resource.update_labels(&mut owner_labels));
        let patch_result = self.api.patch(&object_ref.name, &self.patch_params(), &patch).await;
        if let Err(kube::Error::Api(ErrorResponse { code: 409, .. })) = patch_result {
            let object = self.api.get(&object_ref.name).await?;
            return Err(self.generate_conflict(object_ref, &object));
        }
        patch_result.map_err(|e| Status::from(e))
    }

    fn generate_conflict(&self, object_ref: &ObjectRef<S>, object: &S) -> Status {
        if self.owner_mark.is_owned::<S>(&object) {
            debug!("Resource {:?} is owned by us", object_ref.name);
            Conflict
        } else {
            debug!("Resource {:?} is owned by someone else", object_ref.name);
            let details = OwnerConflictDetails::new(object_ref.name.clone(), object_ref.namespace.clone())
                .with_owner(self.owner_mark.get_resource_owner::<S>(&object));
            NotOwned(details)
        }
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
        let owner_labels: BTreeMap<String, String> = (&self.owner_mark).into();
        labels.extend(owner_labels);
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

    pub async fn force_get(&self, object_ref: &ObjectRef<S>) -> Result<S, Status> {
        let object = self.api.get(&object_ref.name).await?;
        if self.owner_mark.is_owned::<S>(&object) {
            Ok(object)
        } else {
            Err(self.generate_conflict(object_ref, &object))
        }
    }

    pub fn get_forced(&self, object_ref: &ObjectRef<S>) -> Option<Arc<S>> {
        self.reader.get(object_ref)
    }

    fn post_params(&self) -> PostParams {
        PostParams {
            field_manager: Some(self.owner_mark.get_owner_name()),
            ..Default::default()
        }
    }

    fn patch_params(&self) -> PatchParams {
        PatchParams {
            field_manager: Some(self.owner_mark.get_owner_name()),
            ..Default::default()
        }
    }
}

#[async_trait]
impl<H, S> KubernetesResourceWatcher<H, S> for KubernetesResourceManager<S>
where
    S: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
{
    async fn start(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let stream = watcher(api.clone(), (&config.owner_mark).into());
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

        Ok(KubernetesResourceManager::new(
            reader,
            handle,
            api,
            config.namespace,
            config.owner_mark,
        ))
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}
