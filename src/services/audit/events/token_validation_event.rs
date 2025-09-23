use serde::Serialize;

pub struct TokenValidationEvent {
    pub token_id: String,
    pub result: TokenValidationResult,
    pub token_type: String,
}

impl TokenValidationEvent {
    pub fn internal(token_id: String, is_successful: bool, details: String) -> Self {
        Self {
            token_id,
            result: make_result(is_successful, details, "internal".to_string()),
            token_type: "internal".to_string(),
        }
    }

    pub fn external(token_id: String, is_successful: bool, details: String) -> Self {
        Self {
            token_id,
            result: make_result(is_successful, details, "external".to_string()),
            token_type: "external".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenValidationResult {
    Allow(String),
    Deny(String),
}

fn make_result(is_successful: bool, details: String, token_type: String) -> TokenValidationResult {
    if is_successful {
        TokenValidationResult::Allow(token_type)
    } else {
        TokenValidationResult::Deny(details)
    }
}
