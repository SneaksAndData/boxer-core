use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
pub struct AuditEventProperties {
    pub is_final: bool,

    pub action: String,

    pub actor: String,

    pub resource: String,

    pub reason: Reason,

    pub external_token_id: String,

    pub internal_token_id: String,

    decision: Option<Decision>,
}

impl AuditEventProperties {
    pub fn new(decision: Decision) -> Self {
        Self {
            decision: Some(decision),
            ..Default::default()
        }
    }

    pub fn decision(&self) -> String {
        self.decision
            .map(|decision| format!("{:?}", decision))
            .unwrap_or("".to_string())
    }
}

impl Default for AuditEventProperties {
    fn default() -> Self {
        Self {
            is_final: false,
            action: Default::default(),
            actor: Default::default(),
            resource: Default::default(),
            reason: Reason {
                policies: Default::default(),
                errors: Default::default(),
            },
            decision: None,
            external_token_id: Default::default(),
            internal_token_id: Default::default(),
        }
    }
}
