use async_trait::async_trait;

pub mod naive_tree;

pub mod parametrized_matcher;
#[cfg(test)]
mod tests;
pub mod trie_bucket;

#[async_trait]
/// A prefix tree (trie) structure for storing and retrieving values based on keys.
pub trait PrefixTree<K, V> {
    /// Retrieves a value associated with the given key sequence.
    async fn get(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
}

/// A mutable prefix tree that supports insertion and deletion of key-value pairs.
#[async_trait]
pub trait MutablePrefixTree<K, V> {
    /// Inserts a key-value pair into the trie.
    async fn insert(&mut self, key: impl AsRef<[K]> + Send, value: V);

    /// Deletes a key-value pair from the trie and returns the associated value if it existed.
    async fn delete(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
}
