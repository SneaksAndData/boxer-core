use async_trait::async_trait;
use std::sync::Arc;

pub mod hash_bucket;
pub mod request_segment_bucket;

#[async_trait]
pub trait TrieBucket<Key, Value> {
    async fn child(&self, key: &Key) -> Option<Arc<Self>>;

    async fn create_child(&self, key: &Key);

    async fn get_value(&self, key: &Key) -> Option<Value>;

    async fn clear(&self, key: &Key) -> Option<Value>;

    async fn set_value(&self, value: Value, key: &Key);
}
