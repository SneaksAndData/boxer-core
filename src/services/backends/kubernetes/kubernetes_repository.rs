pub mod resource_manager;
pub mod schema_repository;
pub mod soft_delete_resource;
#[cfg(test)]
mod tests;
pub mod to_resource;
mod try_from_resource;
pub mod try_into_object_ref;

use crate::services::backends::kubernetes::kubernetes_repository::resource_manager::ResourceManager;
use crate::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use crate::services::backends::kubernetes::kubernetes_repository::to_resource::ToResource;
use crate::services::backends::kubernetes::kubernetes_repository::try_from_resource::TryFromResource;
use crate::services::backends::kubernetes::kubernetes_repository::try_into_object_ref::TryIntoObjectRef;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::not_found_details::NotFoundDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use crate::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use async_trait::async_trait;
use kube::runtime::reflector::ObjectRef;
use log::{debug, warn};
use std::hash::Hash;
use std::time::Duration;
use tokio::time::Instant;

pub struct KubernetesRepository<R, M>
where
    R: kube::Resource + SoftDeleteResource + Send + Sync + 'static,
    R::DynamicType: Hash + Eq,
    M: ResourceManager<R> + Send + Sync + 'static,
{
    pub resource_manager: M,
    pub operation_timeout: Duration,
    _marker: std::marker::PhantomData<R>,
}

impl<R, M> KubernetesRepository<R, M>
where
    R: kube::Resource + SoftDeleteResource + UpdateLabels + Send + Sync + 'static,
    R::DynamicType: Hash + Eq + Clone + Default,
    M: ResourceManager<R> + Send + Sync + 'static,
{
    pub async fn start(resource_manager: M, operation_timeout: Duration) -> anyhow::Result<Self> {
        Ok(KubernetesRepository::<R, M> {
            resource_manager,
            operation_timeout,
            _marker: std::marker::PhantomData,
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
impl<M, Key, Value, Resource> ReadOnlyRepository<Key, Value> for KubernetesRepository<Resource, M>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + 'static,
    Value: TryFromResource<Resource, Error = Status> + Send + Sync + 'static,
    M: ResourceManager<Resource> + Send + Sync + 'static,
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
impl<M, Key, Value, Resource> CanDelete<Key, Value> for KubernetesRepository<Resource, M>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + Clone + 'static,
    Value: Send + Sync + 'static,
    M: ResourceManager<Resource> + Send + Sync + 'static,
{
    type DeleteError = Status;

    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError> {
        let object_ref = key.try_into_object_ref(self.resource_manager.namespace().clone())?;
        let start_time = Instant::now();
        loop {
            let resource = self.resource_manager.get_uncached(&object_ref).await;
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
impl<Key, Value, Resource, M> UpsertRepository<Key, Value> for KubernetesRepository<Resource, M>
where
    Resource: SoftDeleteResource + UpdateLabels,
    Resource::DynamicType: Hash + Eq + Clone + Default,
    Key: TryIntoObjectRef<Resource, Error = anyhow::Error> + Send + Sync + Clone + 'static,
    Value: ToResource<Resource> + TryFromResource<Resource, Error = Status> + Send + Sync + 'static,
    M: ResourceManager<Resource> + Send + Sync + 'static,
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
