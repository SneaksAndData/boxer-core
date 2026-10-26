use log::{info, warn};

use crate::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use crate::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use crate::services::legacy::prefix_tree::MutablePrefixTree;
use crate::services::legacy::prefix_tree::PrefixTree;
use crate::services::legacy::prefix_tree::naive_tree::NaiveTrie;
use crate::services::legacy::prefix_tree::parametrized_matcher::ParametrizedMatcher;
use crate::services::legacy::prefix_tree::trie_bucket::request_segment_bucket::PrioritizedBucket;
use anyhow::anyhow;
use async_trait::async_trait;
use cedar_policy::EntityUid;
use futures::StreamExt;
use kube::Resource;
use kube::runtime::watcher::Error;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::RwLock;

pub mod backend;
pub mod schema_bound_trie_repository;

pub struct TrieData<Key, Value>
where
    Key: Debug + Send + Sync,
    Value: Send + Sync,
{
    trie: NaiveTrie<PrioritizedBucket<Key, Value>>,
}

struct TrieRepositoryData<Key, Value>
where
    Key: Debug + Send + Sync,
    Value: Send + Sync,
{
    pub rw_lock: RwLock<TrieData<Key, Value>>,
}

impl<Key, Value> TrieRepositoryData<Key, Value>
where
    Key: Ord + Send + Sync + Debug,
    Value: Send + Sync,
{
    pub fn new() -> Self {
        TrieRepositoryData {
            rw_lock: RwLock::new(TrieData {
                trie: NaiveTrie::<PrioritizedBucket<Key, Value>>::new(),
            }),
        }
    }
}

pub trait EntityCollectionResource<Key> {
    fn stream(
        self,
    ) -> impl futures::Stream<Item = Result<(Vec<Key>, EntityUid, bool), anyhow::Error>> + Send + Sync + 'static;
}

pub trait SchemaBoundResource {
    fn schema(&self) -> String;
}

#[async_trait]
impl<Key> ReadOnlyRepository<Vec<Key>, EntityUid> for TrieRepositoryData<Key, EntityUid>
where
    Key: Hash + ParametrizedMatcher + Sync + Send + Debug + Clone + Eq,
{
    type ReadError = anyhow::Error;

    async fn get(&self, key: Vec<Key>) -> Result<EntityUid, Self::ReadError> {
        let guard = self.rw_lock.read().await;
        guard
            .trie
            .get(&key)
            .await
            .map(|e| e.clone())
            .ok_or(anyhow!("Entity not found: {:?}", key))
    }
}

#[async_trait]
impl<Key> UpsertRepository<Vec<Key>, EntityUid> for TrieRepositoryData<Key, EntityUid>
where
    Key: Ord + Send + Sync + Debug + Clone + Hash + ParametrizedMatcher,
{
    type Error = anyhow::Error;

    async fn upsert(&self, key: Vec<Key>, entity: EntityUid) -> Result<EntityUid, Self::Error> {
        let mut guard = self.rw_lock.write().await;
        guard.trie.insert(key, entity.clone()).await;
        Ok(entity)
    }

    async fn exists(&self, key: Vec<Key>) -> Result<bool, Self::Error> {
        let guard = self.rw_lock.read().await;
        Ok(guard.trie.get(key).await.is_some())
    }
}

#[async_trait]
impl<Key> CanDelete<Vec<Key>, EntityUid> for TrieRepositoryData<Key, EntityUid>
where
    Key: Ord + Send + Sync + Debug + Clone + Hash + ParametrizedMatcher,
{
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: Vec<Key>) -> Result<(), Self::DeleteError> {
        let guard = self.rw_lock.write().await;
        guard.trie.delete(key).await;
        Ok(())
    }
}

#[async_trait]
impl<R, K> ResourceUpdateHandler<R> for TrieRepositoryData<K, EntityUid>
where
    R: EntityCollectionResource<K> + Debug + Resource + Send + Sync + 'static,
    K: Ord + Send + Sync + Debug + Clone + Hash + 'static + ParametrizedMatcher,
{
    async fn handle_update(&self, event: Result<R, Error>) -> () {
        match event {
            Err(e) => warn!("Failed to handle update: {:?}", e),
            Ok(resource) => {
                // Using the unwrap method here because the resource should always have a name
                let resource_id = resource.meta().name.clone().unwrap();
                resource
                    .stream()
                    .for_each(|result| {
                        let name = resource_id.clone();
                        async move {
                            match result {
                                Ok((segments, action_uid, active)) => {
                                    let result = if active {
                                        self.upsert(segments.clone(), action_uid.clone()).await.map(|_| ())
                                    } else {
                                        self.delete(segments.clone()).await
                                    };
                                    if let Err(e) = result {
                                        warn!(resource_id = name; "Failed to upsert action: {}", e);
                                    } else {
                                        info!(
                                            resource_id = name;
                                            "Successfully upserted object with key {:?} and UID: {}",
                                            segments, action_uid
                                        );
                                    }
                                }
                                Err(e) => warn!(resource_id = name; "Error processing action route: {}", e),
                            }
                        }
                    })
                    .await;
                info!(resource_id = resource_id; "Finished updating action discovery trie");
            }
        }
    }
}
