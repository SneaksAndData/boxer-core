use serde::{Deserialize, Serialize};

/// [`TokenAuditEvent`] represents the audit information related to a token validation,
/// including the token's ID, the result of the validation, any errors that occurred during
/// validation, and the type of token (internal or external).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAuditEvent {
    pub token_id: String,
}

impl TokenAuditEvent {
    /// Creates a new TokenAuditEvent for an external token validation, with no token ID or errors.
    pub fn external(token_id: impl Into<String>) -> Self {
        Self {
            token_id: token_id.into(),
        }
    }
}
