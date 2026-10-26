use crate::services::legacy::prefix_tree::trie_bucket::TrieBucket;
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A bucket implementation using a hash map to store children and an optional value.
#[derive(Default, Debug)]
pub struct HashTrieBucket<K, V> {
    children: RwLock<HashMap<K, Arc<Self>>>,
    value: RwLock<Option<V>>,
}

impl<K, V> HashTrieBucket<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn new() -> Self {
        HashTrieBucket {
            children: RwLock::new(HashMap::new()),
            value: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<Key, Value> TrieBucket<Key, Value> for HashTrieBucket<Key, Value>
where
    Key: Eq + Hash + Clone + Send + Sync,
    Value: Clone + Send + Sync,
{
    async fn child(&self, key: &Key) -> Option<Arc<Self>> {
        self.children.read().await.get(key).map(|v| v.clone())
    }

    async fn create_child(&self, key: &Key) {
        self.children
            .write()
            .await
            .insert(key.clone(), Arc::new(HashTrieBucket::new()));
    }

    async fn get_value(&self, _key: &Key) -> Option<Value> {
        self.value.read().await.clone()
    }

    async fn clear(&self, _key: &Key) -> Option<Value> {
        self.value.write().await.take()
    }

    async fn set_value(&self, value: Value, _key: &Key) {
        self.value.write().await.replace(value);
    }
}
