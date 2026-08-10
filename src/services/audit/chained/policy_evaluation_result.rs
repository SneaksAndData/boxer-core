use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
use serde::{Deserialize, Serialize};

/// Describes the Cedar policy evaluation result in the event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub action: Option<String>,
    pub actor: Option<String>,
    pub resource: Option<String>,
    pub reason: Option<Reason>,

    pub decision: Decision,
}

impl PolicyEvaluationResult {
    pub(crate) fn from_result(
        action: String,
        actor: String,
        resource: String,
        reason: Reason,
        decision: Decision,
    ) -> PolicyEvaluationResult {
        Self {
            action: Some(action),
            actor: Some(actor),
            resource: Some(resource),
            reason: Some(reason),
            decision,
        }
    }

    pub fn empty_deny() -> Self {
        Self {
            action: None,
            actor: None,
            resource: None,
            reason: None,
            decision: Decision::Deny,
        }
    }
}
