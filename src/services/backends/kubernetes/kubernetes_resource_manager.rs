pub mod object_owner_mark;
pub mod status;
#[cfg(test)]
mod tests;

use crate::services::backends::kubernetes::kubernetes_repository::resource_manager::ResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::owner_conflict_details::OwnerConflictDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status::{Conflict, NotOwned};
use crate::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams};
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

pub struct GenericKubernetesResourceManager<R>
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

impl<S> GenericKubernetesResourceManager<S>
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
        GenericKubernetesResourceManager {
            reader,
            handle,
            api,
            namespace,
            owner_mark,
        }
    }

    fn generate_conflict(&self, object_ref: &ObjectRef<S>, object: &S) -> Status {
        if self.owner_mark.is_owned::<S>(&object) {
            debug!("Resource {:?} is owned by us", object_ref.name);
            Conflict
        } else {
            debug!("Resource {:?} is owned by someone else", object_ref.name);
            let details =
                OwnerConflictDetails::from(object_ref).with_owner(self.owner_mark.get_resource_owner::<S>(&object));
            NotOwned(details)
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
impl<R> ResourceManager<R> for GenericKubernetesResourceManager<R>
where
    R: UpdateLabels + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    async fn get_uncached(&self, object_ref: &ObjectRef<R>) -> Result<R, Status> {
        let object = self.api.get(&object_ref.name).await?;
        if self.owner_mark.is_owned::<R>(&object) {
            Ok(object)
        } else {
            Err(self.generate_conflict(object_ref, &object))
        }
    }
    async fn upsert(&self, object_ref: &ObjectRef<R>, resource: R) -> Result<R, Status> {
        let mut owner_labels: BTreeMap<String, String> = (&self.owner_mark).into();
        let patch = Patch::Apply(resource.update_labels(&mut owner_labels));
        let patch_result = self.api.patch(&object_ref.name, &self.patch_params(), &patch).await;
        if let Err(kube::Error::Api(ErrorResponse { code: 409, .. })) = patch_result {
            let object = self.api.get(&object_ref.name).await?;
            return Err(self.generate_conflict(object_ref, &object));
        }
        patch_result.map_err(|e| Status::from(e))
    }
    fn get(&self, object_ref: &ObjectRef<R>) -> Result<Arc<R>, Status> {
        let result = self.reader.get(object_ref);
        match result {
            None => Err(Status::NotFound(object_ref.into())),
            Some(resource) => Ok(resource),
        }
    }

    fn namespace(&self) -> String {
        self.namespace.clone()
    }
}

#[async_trait]
impl<H, S> KubernetesResourceWatcher<H, S> for GenericKubernetesResourceManager<S>
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

        Ok(GenericKubernetesResourceManager::new(
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
