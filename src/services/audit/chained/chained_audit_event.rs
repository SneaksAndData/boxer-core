use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
use serde::{Deserialize, Serialize};

/// [`ChainedAuditEvent`] represents the information collected during the processing of a
/// request that is relevant for auditing purposes. It includes details about the external and
/// internal token validation, the action being performed, the actor, the resource, the
/// decision made by the authorization engine, and any reasons for that decision.
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
    pub fn empty() -> ChainedAuditEvent {
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
