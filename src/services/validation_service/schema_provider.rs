use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::Schema;

/// Reads the schema from the external storage.
#[async_trait]
pub trait SchemaProvider: Send + Sync {
    /// Returns the schema with the specified name.
    async fn get_schema(&self, name: String) -> Result<Schema>;
}
