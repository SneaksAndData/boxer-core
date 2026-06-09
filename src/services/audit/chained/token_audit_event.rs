use crate::services::audit::events::token_validation_event::TokenValidationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// [`TokenAuditEvent`] represents the audit information related to a token validation,
/// including the token's ID, the result of the validation, any errors that occurred during
/// validation, and the type of token (internal or external).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAuditEvent {
    pub token_id: Option<String>,
    pub result: Option<TokenValidationResult>,
    pub reason_errors: HashSet<String>,
    pub token_type: Option<String>,
}

impl TokenAuditEvent {
    /// Creates a new TokenAuditEvent for an external token validation, with no token ID or errors.
    pub fn external() -> Self {
        Self {
            token_id: None,
            result: None,
            reason_errors: HashSet::new(),
            token_type: Some("external".into()),
        }
    }

    /// Adds a token ID to the TokenAuditEvent by computing the MD5 hash of the provided token string.
    pub fn with_token_id(mut self, token: &str) -> Self {
        let token_hash = md5::compute(token);
        self.token_id = Some(format!("md5:{:x}", token_hash));
        self
    }

    /// Adds an error to the TokenAuditEvent, setting the token ID to a placeholder value and the
    /// result to Deny. This method should be used in external token validation when the token is
    /// not provided or invalid.
    pub fn failure(mut self, error: actix_web::Error) -> Self {
        self.token_id = Some("token-not-provided".into());
        self.result = Some(TokenValidationResult::Deny);
        self.reason_errors.insert(error.to_string());
        self
    }
}
