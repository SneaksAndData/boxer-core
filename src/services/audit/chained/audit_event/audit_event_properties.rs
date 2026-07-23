use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
pub struct AuditEventProperties {
    pub is_final: bool,

    pub action: String,

    pub actor: String,

    pub resource: String,

    pub reason: Reason,

    pub decision: Decision,

    pub external_token_id: String,

    pub internal_token_id: String,
}

impl Default for AuditEventProperties {
    fn default() -> Self {
        Self {
            is_final: Default::default(),
            action: Default::default(),
            actor: Default::default(),
            resource: Default::default(),
            reason: Reason {
                policies: Default::default(),
                errors: Default::default(),
            },
            decision: Decision::Deny,
            external_token_id: Default::default(),
            internal_token_id: Default::default(),
        }
    }
}
