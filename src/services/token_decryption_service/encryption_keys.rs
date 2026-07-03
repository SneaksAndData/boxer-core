use serde::Deserialize;
use std::collections::HashMap;

/// Configuration settings that contains the encryption keys used by Boxer validator app.
#[derive(Debug, Deserialize)]
pub struct EncryptionKeys(HashMap<String, String>);

impl EncryptionKeys {
    /// Returns the encryption key by key id
    pub fn get(&self, key_id: &str) -> Option<&String> {
        self.0.get(key_id)
    }
}
