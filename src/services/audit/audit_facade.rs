pub mod to_audit_record;

use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use crate::services::audit::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::resource_modification_audit_event::{ModificationResult, ResourceModificationAuditEvent};
use crate::services::audit::AuditService;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};
use async_trait::async_trait;
use std::marker::PhantomData;
use std::sync::Arc;

pub trait WithAuditFacade<Key, Value> {
    type Error;

    fn with_audit(
        self: Arc<Self>,
        audit_service: Arc<dyn AuditService>,
    ) -> Arc<
        dyn UpsertRepositoryWithDelete<
                Key,
                Value,
                Error = Self::Error,
                DeleteError = Self::Error,
                ReadError = Self::Error,
            >,
    >
    where
        Self: Sized + Send + Sync + 'static;
}

impl<Repo, Key, Value> WithAuditFacade<Key, Value> for Repo
where
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>
        + Send
        + Sync
        + 'static,
    Key: ToAuditRecord + Send + Sync + 'static,
    Value: ToAuditRecord + Send + Sync + 'static,
{
    type Error = Status;

    fn with_audit(
        self: Arc<Self>,
        audit_service: Arc<dyn AuditService>,
    ) -> Arc<
        dyn UpsertRepositoryWithDelete<
                Key,
                Value,
                Error = Self::Error,
                DeleteError = Self::Error,
                ReadError = Self::Error,
            >,
    > {
        let resource_type = std::any::type_name::<Value>().to_string();
        Arc::new(AuditFacade {
            audit_service,
            resource_type,
            underlying: self,
            _p: Default::default(),
        })
    }
}

struct AuditFacade<Repo, Key, Value>
where
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>,
    Key: Send + Sync + 'static,
    Value: Send + Sync + 'static,
{
    audit_service: Arc<dyn AuditService>,
    resource_type: String,
    underlying: Arc<Repo>,
    _p: PhantomData<(Key, Value)>,
}

#[async_trait]
impl<Repo, Key, Value> UpsertRepository<Key, Value> for AuditFacade<Repo, Key, Value>
where
    Key: ToAuditRecord + Send + Sync + 'static,
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>,
    Value: ToAuditRecord + Send + Sync + 'static,
{
    type Error = Status;

    async fn upsert(&self, key: Key, entity: Value) -> Result<Value, Self::Error> {
        let id = key.to_audit_record();

        let result = self.underlying.upsert(key, entity).await;

        let event =
            ResourceModificationAuditEvent::new(id, self.resource_type.clone(), ModificationResult::from(&result));
        self.audit_service.record_resource_modification(event)?;
        result
    }

    async fn exists(&self, key: Key) -> Result<bool, Self::Error> {
        self.underlying.exists(key).await
    }
}

#[async_trait]
impl<Repo, Key, Value> ReadOnlyRepository<Key, Value> for AuditFacade<Repo, Key, Value>
where
    Key: ToAuditRecord + Send + Sync + 'static,
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>,
    Value: 'static + Send + Sync,
{
    type ReadError = Status;

    async fn get(&self, key: Key) -> Result<Value, Self::ReadError> {
        self.underlying.get(key).await
    }
}

#[async_trait]
impl<Repo, Key, Value> CanDelete<Key, Value> for AuditFacade<Repo, Key, Value>
where
    Key: ToAuditRecord + Send + Sync + 'static,
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>,
    Value: 'static + Send + Sync,
{
    type DeleteError = Status;

    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError> {
        let id = key.to_audit_record();
        let result = self.underlying.delete(key).await;
        let event = ResourceDeleteAuditEvent::new(id, self.resource_type.clone(), result.is_ok());
        self.audit_service.record_resource_deletion(event)?;
        result
    }
}

#[async_trait]
impl<Repo, Key, Value> UpsertRepositoryWithDelete<Key, Value> for AuditFacade<Repo, Key, Value>
where
    Repo: UpsertRepositoryWithDelete<Key, Value, Error = Status, DeleteError = Status, ReadError = Status>,
    Key: ToAuditRecord + Send + Sync + 'static,
    Value: ToAuditRecord + Send + Sync + 'static,
{
}
