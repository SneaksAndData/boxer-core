use serde::Serialize;

pub struct TokenValidationEvent {
    pub token_id: String,
    pub result: TokenValidationResult,
    pub token_type: String,
}

impl TokenValidationEvent {
    pub fn internal(token_id: String, result: TokenValidationResult) -> Self {
        Self {
            token_id,
            result,
            token_type: "internal".to_string(),
        }
    }

    pub fn external(token_id: String, result: TokenValidationResult) -> Self {
        Self {
            token_id,
            result,
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
