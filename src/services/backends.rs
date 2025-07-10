use crate::services::base::types::SchemaRepository;
use async_trait::async_trait;
use std::sync::Arc;

pub mod kubernetes;
pub mod memory;

#[allow(dead_code)]
pub trait Backend: SchemaRepositorySource + Send + Sync {}

pub trait SchemaRepositorySource: Send + Sync {
    #[allow(dead_code)]
    fn get_schemas_repository(&self) -> Arc<SchemaRepository>;
}

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
