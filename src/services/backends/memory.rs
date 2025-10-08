use crate::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, ReadOnlyRepositoryWithFactory, UpsertRepository, UpsertRepositoryWithDelete,
    ValueFactory,
};
use anyhow::bail;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::RwLock;

#[async_trait]
impl<Key, Entity> ReadOnlyRepository<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Clone + Send + Sync,
    Key: Debug + Eq + Hash + Send + Sync,
{
    type ReadError = anyhow::Error;

    async fn get(&self, key: Key) -> Result<Entity, Self::ReadError> {
        let read_guard = self.read().await;
        match (*read_guard).get(&key) {
            Some(entity) => Ok(entity.clone()),
            None => bail!("Entity {:?} not found", key),
        }
    }
}

#[async_trait]
impl<Key, Entity> UpsertRepository<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug,
{
    type Error = anyhow::Error;

    async fn upsert(&self, key: Key, entity: Entity) -> Result<Entity, Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).insert(key, entity.clone());
        Ok(entity)
    }

    async fn exists(&self, key: Key) -> Result<bool, Self::Error> {
        let read_guard = self.read().await;
        Ok((*read_guard).get(&key).is_some())
    }
}

#[async_trait]
impl<Key, Entity> CanDelete<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug,
{
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: Key) -> Result<(), Self::DeleteError> {
        let mut write_guard = self.write().await;
        (*write_guard).remove(&key);
        Ok(())
    }
}

impl<Key, Entity> UpsertRepositoryWithDelete<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug,
{
}

#[async_trait]
impl<Key, Entity> ReadOnlyRepositoryWithFactory<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug + Clone,
{
    type ReadError = anyhow::Error;

    async fn get(
        &self,
        key: Key,
        value_factory: &dyn ValueFactory<Key, Entity, CreateError = Self::ReadError>,
    ) -> Result<Entity, Self::ReadError> {
        // First, acquire a read lock to check if the entity exists.
        {
            let read_guard = self.read().await;
            if let Some(entity) = (*read_guard).get(&key) {
                return Ok(entity.clone());
            }
        }
        // Release the read lock before calling the factory.
        let new_entity = value_factory.create(&key).await?;
        // Acquire a write lock to insert the new entity.
        let mut write_guard = self.write().await;
        // Check again in case another thread inserted it while we were creating.
        if let Some(entity) = (*write_guard).get(&key) {
            Ok(entity.clone())
        } else {
            (*write_guard).insert(key, new_entity.clone());
            Ok(new_entity)
        }
    }
}
