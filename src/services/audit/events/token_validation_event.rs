use serde::Serialize;
use std::collections::HashSet;

pub struct TokenValidationEvent {
    pub token_id: String,
    pub result: TokenValidationResult,
    pub reason_errors: HashSet<String>,
    pub token_type: String,
}

impl TokenValidationEvent {
    pub fn internal(token_id: String, is_successful: bool, details: HashSet<String>) -> Self {
        Self {
            token_id,
            result: make_result(is_successful),
            reason_errors: details,
            token_type: "internal".to_string(),
        }
    }

    pub fn external(token_id: String, is_successful: bool, details: HashSet<String>) -> Self {
        Self {
            token_id,
            result: make_result(is_successful),
            reason_errors: details,
            token_type: "external".to_string(),
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
