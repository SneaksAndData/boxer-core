use std::sync::Arc;
use async_trait::async_trait;
use crate::services::base::types::SchemaRepository;

pub mod kubernetes;
pub mod memory;

#[allow(dead_code)]
pub trait Backend: SchemaRepositorySource + Send + Sync {

}

pub trait SchemaRepositorySource: Send + Sync {
    #[allow(dead_code)]
    fn get_schemas_repository(&self) -> Arc<SchemaRepository>;
}

#[async_trait]
#[allow(dead_code)]
pub trait BackendConfiguration: Send + Sync + Sized {
    type BackendSettings;
    
    type InitializedBackend: Backend;

    async fn configure(mut self, cm: &Self::BackendSettings, instance_name: String) -> anyhow::Result<Arc<Self::InitializedBackend>>;
}
