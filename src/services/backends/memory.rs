use crate::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use anyhow::bail;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::RwLock;

#[async_trait]
impl<Entity, Key> ReadOnlyRepository<Key, Entity> for RwLock<HashMap<Key, Entity>>
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
impl<Entity, Key> UpsertRepository<Key, Entity> for RwLock<HashMap<Key, Entity>>
where
    Entity: Send + Sync + Clone,
    Key: Send + Sync + Eq + Hash + Debug,
{
    type Error = anyhow::Error;

    async fn upsert(&self, key: Key, entity: Entity) -> Result<(), Self::Error> {
        let mut write_guard = self.write().await;
        (*write_guard).insert(key, entity);
        Ok(())
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
