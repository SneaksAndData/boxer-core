use async_trait::async_trait;

#[async_trait]
/// Represents a repository for policies
pub trait UpsertRepository<Key, Entity>: Send + Sync {
    type Error;

    /// Retrieves a policy by id
    async fn get(&self, key: Key) -> Result<Entity, Self::Error>;

    /// Updates or inserts a policy by id
    async fn upsert(&self, key: Key, entity: Entity) -> Result<(), Self::Error>;

    /// Deletes policy by id
    async fn delete(&self, key: Key) -> Result<(), Self::Error>;

    /// Checks if an object exists
    async fn exists(&self, key: Key) -> Result<bool, Self::Error>;
}