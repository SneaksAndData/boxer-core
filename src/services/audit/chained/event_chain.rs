use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,

    pub action: Option<String>,
    pub actor: Option<String>,
    pub resource: Option<String>,
    pub decision: Option<Decision>,
    pub reason: Option<Reason>,
}

impl ChainedAuditEvent {
    pub(crate) fn set_external_token_id(&mut self, token: &str) {
        self.external_token = Some(TokenAuditEvent::external().with_token_id(token));
    }

    pub(crate) fn set_external_token_error(&mut self, error: actix_web::Error) {
        self.external_token = Some(TokenAuditEvent::external().failure(error));
    }
}

impl ChainedAuditEvent {
    pub fn new() -> ChainedAuditEvent {
        ChainedAuditEvent {
            external_token: None,
            internal_token: None,
            action: None,
            actor: None,
            resource: None,
            decision: None,
            reason: None,
        }
    }
}
