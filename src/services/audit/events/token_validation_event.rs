use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct TokenValidationEvent {
    pub token_id: String,
    pub result: TokenValidationResult,
    pub reason_errors: HashSet<String>,
    pub token_type: String,
    pub token_metadata: Option<TokenMetadata>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct TokenMetadata {
    exp: Option<u64>,
    nbf: Option<u64>,
    sub: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
}

impl TokenValidationEvent {
    pub fn internal(token: &str, is_successful: bool, details: HashSet<String>) -> Self {
        let token_hash = md5::compute(token);
        Self {
            token_id: format!("md5:{:x}", token_hash),
            result: make_result(is_successful),
            reason_errors: details,
            token_type: "internal".to_string(),
            token_metadata: None,
        }
    }

    pub fn external(token: &str, is_successful: bool, details: HashSet<String>) -> Self {
        let metadata = jsonwebtoken::dangerous::insecure_decode::<TokenMetadata>(token);
        warn!("Decoded token metadata: {:?}", metadata);
        let token_hash = md5::compute(token);
        Self {
            token_id: format!("md5:{:x}", token_hash),
            result: make_result(is_successful),
            reason_errors: details,
            token_type: "external".to_string(),
            token_metadata: metadata.ok().map(|data| data.claims),
        }
    }

    pub fn external_empty(is_successful: bool, details: HashSet<String>) -> Self {
        Self {
            token_id: "token-not-provided".to_string(),
            result: make_result(is_successful),
            reason_errors: details,
            token_type: "external".to_string(),
            token_metadata: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenValidationResult {
    Allow,
    Deny,
}

fn make_result(is_successful: bool) -> TokenValidationResult {
    if is_successful {
        TokenValidationResult::Allow
    } else {
        TokenValidationResult::Deny
    }
}
