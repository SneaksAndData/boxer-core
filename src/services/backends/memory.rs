use crate::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, ReadOnlyRepositoryWithFactory, UpsertRepository, UpsertRepositoryWithDelete,
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
    Key: Send + Sync + Eq + Hash + Debug,
{
    type ReadError = anyhow::Error;

    async fn get<F>(&self, key: Key, create_new: F) -> Result<Entity, Self::ReadError>
    where
        F: FnOnce(&Key) -> Entity + Send,
    {
        let mut write_guard = self.write().await;
        if let Some(entity) = (*write_guard).get(&key) {
            Ok(entity.clone())
        } else {
            let new_entity = create_new(&key);
            (*write_guard).insert(key, new_entity.clone());
            Ok(new_entity)
        }
    }
}
