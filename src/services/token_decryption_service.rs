mod encryption_keys;
mod token_settings;

use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::services::token_decryption_service::encryption_keys::EncryptionKeys;
use crate::services::token_decryption_service::token_settings::TokenValidationSettings;
use anyhow::{Result, anyhow};
use collection_macros::hashset;
use josekit::jwe::Dir;
use josekit::jwt;
use serde_json::Value;
use std::collections::HashSet;

pub struct TokenDecryptionService {
    pub keys: EncryptionKeys,
    pub valid_issuers: HashSet<String>,
    pub valid_audiences: HashSet<String>,
}

impl TokenDecryptionService {
    pub fn new(keys: EncryptionKeys, token_settings: TokenValidationSettings) -> Self {
        TokenDecryptionService {
            keys,
            valid_issuers: hashset! {token_settings.issuer},
            valid_audiences: hashset![token_settings.audience],
        }
    }
}

impl Decryptor for TokenDecryptionService {
    fn decrypt(&self, encrypted_token: EncryptedToken) -> Result<DynamicClaimsCollection> {
        let raw: String = encrypted_token.into();
        let header = jwt::decode_header(&raw).map_err(anyhow::Error::from)?;
        let key_id = header
            .claim("kid")
            .and_then(Value::as_str)
            .ok_or(anyhow::anyhow!("No 'kid' in token header"))?
            .to_string();

        let key = self
            .keys
            .get(&key_id)
            .ok_or(anyhow!("No key found for kid: {}", key_id))
            .map_err(anyhow::Error::from)?;

        let decrypter = Dir.decrypter_from_bytes(key).map_err(anyhow::Error::from)?;
        let (payload, header) = jwt::decode_with_decrypter(&raw, &decrypter)?;

        let audiences = header.audience().ok_or(anyhow::anyhow!("No audience"))?;
        if !audiences.iter().any(|a| self.valid_audiences.contains(*a)) {
            return Err(anyhow::anyhow!("No valid audience"));
        }

        let issuer = header.issuer().ok_or(anyhow::anyhow!("No issuer"))?;
        if !self.valid_issuers.contains(issuer) {
            return Err(anyhow::anyhow!("Invalid issuer"));
        }

        Ok(payload)
    }
}
