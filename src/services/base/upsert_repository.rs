use async_trait::async_trait;

#[async_trait]
/// Represents a repository for policies
pub trait UpsertRepository<Key, Entity>: ReadOnlyRepository<Key, Entity> + Send + Sync {
    type Error;

    /// Updates or inserts a policy by id
    async fn upsert(&self, key: Key, entity: Entity) -> Result<(), Self::Error>;

    /// Checks if an object exists
    async fn exists(&self, key: Key) -> Result<bool, Self::Error>;
}

#[async_trait]
/// Represents a repository for policies
pub trait ReadOnlyRepository<Key, Entity>: Send + Sync {
    type ReadError;

    /// Retrieves a policy by id
    async fn get(&self, key: Key) -> Result<Entity, Self::ReadError>;
}

#[async_trait]
/// Represents a repository for policies
pub trait CanDelete<Key, Entity>: Send + Sync {
    type DeleteError;

    /// Retrieves a policy by id
    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError>;
}


pub trait UpsertRepositoryWithDelete<Key, Entity>: UpsertRepository<Key, Entity> + CanDelete<Key, Entity> {
    // This trait is a marker trait that combines UpsertRepository and CanDelete
}