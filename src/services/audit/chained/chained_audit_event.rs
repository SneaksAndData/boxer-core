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
    /// Creates a new empty `ChainedAuditEvent` with all fields set to `None`.
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

    /// Checks if the `ChainedAuditEvent` is empty
    pub fn is_empty(&self) -> bool {
        self.external_token.is_none()
            && self.internal_token.is_none()
            && self.action.is_none()
            && self.actor.is_none()
            && self.resource.is_none()
            && self.decision.is_none()
            && self.reason.is_none()
    }
}
