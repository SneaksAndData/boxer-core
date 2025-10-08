use async_trait::async_trait;

#[async_trait]
/// Represents a repository for Boxer entities that can be created or updated
pub trait UpsertRepository<Key, Entity>: ReadOnlyRepository<Key, Entity> + Send + Sync {
    type Error;

    /// Updates or inserts a Boxer entity by id
    async fn upsert(&self, key: Key, entity: Entity) -> Result<Entity, Self::Error>;

    /// Checks if an object exists
    async fn exists(&self, key: Key) -> Result<bool, Self::Error>;
}

#[async_trait]
/// Represents a repository for Boxer entities that can only be read
pub trait ReadOnlyRepository<Key, Entity>: Send + Sync {
    type ReadError;

    /// Retrieves a policy by id
    async fn get(&self, key: Key) -> Result<Entity, Self::ReadError>;
}

#[async_trait]
/// Factory trait to create new entities if they do not exist in the repository
pub trait ValueFactory<Key, Entity>: Send + Sync {
    type CreateError;

    async fn create(&self, key: &Key) -> Result<Entity, Self::CreateError>;
}

#[async_trait]
/// Represents a repository for Boxer entities that can be read
/// with the ability to create new entities if they do not exist
pub trait ReadOnlyRepositoryWithFactory<Key, Entity>: Send + Sync {
    type ReadError;

    /// Retrieves or creates a Boxer entity by id
    async fn get(
        &self,
        key: Key,
        create_new: &dyn ValueFactory<Key, Entity, CreateError = Self::ReadError>,
    ) -> Result<Entity, Self::ReadError>;
}

#[async_trait]
/// Represents a repository for a Boxer entities that can be deleted
pub trait CanDelete<Key, Entity>: Send + Sync {
    type DeleteError;

    /// Retrieves a policy by id
    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError>;
}

/// Combines UpsertRepository and CanDelete traits
pub trait UpsertRepositoryWithDelete<Key, Entity>: UpsertRepository<Key, Entity> + CanDelete<Key, Entity> {
    // This trait is a marker trait that combines UpsertRepository and CanDelete
}
