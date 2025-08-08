use crate::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use crate::services::service_provider::ServiceProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub mod kubernetes;
pub mod memory;

pub trait Backend: ServiceProvider<Arc<SchemaRepository>> + Send + Sync {}

#[async_trait]
#[allow(dead_code)]
pub trait BackendConfiguration: Send + Sync + Sized {
    type BackendSettings;

    type InitializedBackend: Backend;

    async fn configure(
        mut self,
        cm: &Self::BackendSettings,
        instance_name: String,
    ) -> anyhow::Result<Arc<Self::InitializedBackend>>;
}
