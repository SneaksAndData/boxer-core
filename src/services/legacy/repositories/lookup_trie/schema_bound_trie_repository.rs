use crate::services::legacy::repositories::lookup_trie::{
    EntityCollectionResource, SchemaBoundResource, TrieRepositoryData,
};
use anyhow::anyhow;

use crate::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::legacy::prefix_tree::parametrized_matcher::ParametrizedMatcher;
use async_trait::async_trait;
use cedar_policy::EntityUid;
use kube::Resource;
use kube::runtime::watcher;
use log::{info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::RwLock;

pub struct SchemaBoundedTrieRepositoryData<Key>
where
    Key: Ord + Debug + Send + Sync,
{
    buckets: RwLock<HashMap<String, TrieRepositoryData<Key, EntityUid>>>,
}

impl<Key> SchemaBoundedTrieRepositoryData<Key>
where
    Key: Ord + Debug + Send + Sync,
{
    pub fn new() -> Self {
        SchemaBoundedTrieRepositoryData {
            buckets: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl<Key> ReadOnlyRepository<(String, Vec<Key>), EntityUid> for SchemaBoundedTrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug + Hash + 'static + ParametrizedMatcher + Clone,
{
    type ReadError = anyhow::Error;

    async fn get(&self, key: (String, Vec<Key>)) -> Result<EntityUid, Self::ReadError> {
        let (schema, segments) = key;
        let guard = self.buckets.read().await;
        let bucket = guard.get(&schema);
        match bucket {
            Some(trie_data) => trie_data.get(segments).await,
            None => Err(anyhow!("Schema [{:?}] not found for key: [{:?}]", schema, segments)),
        }
    }
}

#[async_trait]
impl<R, Key> ResourceUpdateHandler<R> for SchemaBoundedTrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug + Hash + Clone + 'static + ParametrizedMatcher,
    R: SchemaBoundResource + Resource + EntityCollectionResource<Key> + Send + Sync + Debug + 'static,
{
    async fn handle_update(&self, result: Result<R, watcher::Error>) -> () {
        match &result {
            Ok(document) => {
                let mut guard = self.buckets.write().await;
                info!("Handling update for schema: {}", document.schema());
                let bucket = guard.entry(document.schema()).or_insert_with(TrieRepositoryData::new);
                bucket.handle_update(result).await;
            }
            Err(e) => {
                warn!("Error handling update: {:?}", e);
            }
        }
    }
}
