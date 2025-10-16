use crate::services::backends::kubernetes::kubernetes_resource_manager::spin_lock::SpinLockKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::not_found_details::NotFoundDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, UpdateLabels,
};
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::services::backends::kubernetes::repositories::try_into_object_ref::TryIntoObjectRef;
use crate::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use async_trait::async_trait;
use k8s_openapi::NamespaceResourceScope;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use log::{debug, warn};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

pub mod schema_repository;
mod tests;
pub mod try_into_object_ref;

pub trait SoftDeleteResource:
    kube::Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
    fn is_deleted(&self) -> bool;
    fn set_deleted(&mut self);
    fn clear_managed_fields(&mut self);
}

pub trait ToResource<R>
where
    R: SoftDeleteResource,
{
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<R, Status>;
    fn to_resource_default(&self, object_ref: &ObjectRef<R>) -> Result<R, Status> {
        let object_meta = ObjectMeta {
            name: Some(object_ref.name.clone()),
            namespace: object_ref.namespace.clone(),
            ..Default::default()
        };
        self.to_resource(&object_meta)
    }
}

pub trait TryFromResource<R>
where
    R: kube::Resource + Send + Sync + 'static,
{
    type Error;
    fn try_from_resource(resource: Arc<R>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub struct KubernetesRepository<Resource>
where
    Resource: kube::Resource + SoftDeleteResource + Send + Sync + 'static,
    Resource::DynamicType: Hash + Eq,
{
    pub resource_manager: SpinLockKubernetesResourceManager<Resource>,
    pub operation_timeout: Duration,
}

impl<R> KubernetesRepository<R>
where
    R: kube::Resource + SoftDeleteResource + UpdateLabels + Send + Sync + 'static,
    R::DynamicType: Hash + Eq + Clone + Default,
{
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let operation_timeout = config.operation_timeout;
        let resource_manager = SpinLockKubernetesResourceManager::start(config, Arc::new(LoggingUpdateHandler)).await?;
        Ok(KubernetesRepository {
            resource_manager,
            operation_timeout,
        })
    }

    pub fn namespace(&self) -> String {
        self.resource_manager.namespace()
    }

    pub async fn try_delay(&self, start_time: Instant, resource: &ObjectRef<R>, operation: &str) -> Result<(), Status> {
        if start_time.elapsed() > self.operation_timeout {
            let message = format!(
                "Timed out after {:?} waiting for {:?} resource: {:?}/{:?}",
                self.operation_timeout, operation, resource.namespace, resource.name
            );
            return Err(Status::Timeout(message));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

#[async_trait]
impl<Key, Value, Resource> ReadOnlyRepository<Key, Value> for KubernetesRepository<Resource>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + 'static,
    Value: TryFromResource<Resource, Error = Status> + Send + Sync + 'static,
{
    type ReadError = Status;

    async fn get(&self, key: Key) -> Result<Value, Self::ReadError> {
        let object_ref = key.try_into_object_ref(self.resource_manager.namespace().clone())?;
        let resource = self.resource_manager.get(&object_ref);
        match resource {
            Ok(resource) => {
                if resource.is_deleted() {
                    return Err(Status::Deleted(NotFoundDetails::from(&object_ref)));
                }
                let value: Value = Value::try_from_resource(resource.clone())?;
                Ok(value)
            }
            Err(other) => Err(other),
        }
    }
}

#[async_trait]
impl<Key, Value, Resource> CanDelete<Key, Value> for KubernetesRepository<Resource>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + Clone + 'static,
    Value: Send + Sync + 'static,
{
    type DeleteError = Status;

    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError> {
        let object_ref = key.try_into_object_ref(self.resource_manager.namespace().clone())?;
        let start_time = Instant::now();
        loop {
            let resource = self.resource_manager.force_get(&object_ref).await;
            if let Err(e) = resource {
                return Err(e);
            }

            if let Ok(r) = resource {
                let mut r = r;
                if r.is_deleted() {
                    return Err(Status::Deleted(NotFoundDetails::from(&object_ref)));
                }
                r.set_deleted();
                r.clear_managed_fields();
                let upsert_result = self.resource_manager.upsert(&object_ref, r.clone()).await;
                match upsert_result {
                    Ok(_) => return Ok(()),
                    Err(Status::NotOwned(details)) => {
                        warn!("Object is not owned by us: {:?}", details);
                        return Err(Status::NotOwned(details));
                    }
                    Err(e) => warn!("Recoverable error during upsert operation, retrying: {:?}", e),
                }
            }
            self.try_delay(start_time, &object_ref, "delete").await?;
        }
    }
}

#[async_trait]
impl<Key, Value, Resource> UpsertRepository<Key, Value> for KubernetesRepository<Resource>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + Clone + 'static,
    Value: ToResource<Resource> + TryFromResource<Resource, Error = Status> + Send + Sync + 'static,
{
    type Error = Status;

    async fn upsert(&self, key: Key, entity: Value) -> Result<Value, Self::Error> {
        let start_time = Instant::now();
        let object_ref = key.try_into_object_ref(self.resource_manager.namespace().clone())?;
        loop {
            let resource = self.resource_manager.get(&object_ref);

            match resource {
                Err(e) => {
                    if e.is_not_found() {
                        let new = entity.to_resource_default(&object_ref)?;
                        let upsert_result = self.resource_manager.upsert(&object_ref, new).await;
                        match upsert_result {
                            Ok(_) => return Ok(entity),
                            Err(Status::Conflict) => self.try_delay(start_time, &object_ref, "upsert").await?,
                            Err(Status::NotOwned(details)) => {
                                debug!("Owner conflict: {:?}", details);
                                return Err(Status::NotOwned(details));
                            }
                            Err(e) => return Err(e),
                        }
                    } else {
                        return Err(e);
                    }
                }
                Ok(r) => {
                    if r.is_deleted() {
                        return Err(Status::Deleted(NotFoundDetails::from(&object_ref)));
                    }
                    let mut meta_mut = r.meta().clone();
                    meta_mut.managed_fields = None;
                    let new = entity.to_resource(&meta_mut)?;
                    let upsert_result = self.resource_manager.upsert(&object_ref, new).await;
                    match upsert_result {
                        Ok(_) => return Ok(entity),
                        Err(Status::Conflict) => self.try_delay(start_time, &object_ref, "upsert").await?,
                        Err(e) => return Err(e),
                    }
                }
            }
            self.try_delay(start_time, &object_ref, "upsert").await?;
        }
    }

    async fn exists(&self, key: Key) -> Result<bool, Self::Error> {
        let object_ref = key.try_into_object_ref(self.resource_manager.namespace().clone())?;
        Ok(self.resource_manager.get(&object_ref).is_ok())
    }
}
