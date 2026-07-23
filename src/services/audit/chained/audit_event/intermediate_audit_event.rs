use crate::services::audit::chained::audit_event::audit_event_properties::AuditEventProperties;
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;

#[derive(Clone, Debug)]
pub struct IntermediateAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,
}

impl IntermediateAuditEvent {
    pub fn is_empty(&self) -> bool {
        self.external_token.is_none() && self.internal_token.is_none()
    }
}

impl IntermediateAuditEvent {
    pub fn get_properties(&self) -> AuditEventProperties {
        let mut properties = AuditEventProperties {
            is_final: false,
            ..Default::default()
        };

        properties.external_token_id = self
            .external_token
            .as_ref()
            .and_then(|token| token.token_id.clone())
            .unwrap_or_default();
        properties.internal_token_id = self
            .internal_token
            .as_ref()
            .and_then(|token| token.token_id.clone())
            .unwrap_or_default();

        properties
    }
}

impl IntermediateAuditEvent {
    pub fn finalize(&mut self, result: PolicyEvaluationResult) -> FinalAuditEvent {
        FinalAuditEvent {
            external_token: self.external_token.take(),
            internal_token: self.internal_token.take(),
            policy_evaluation_result: result,
        }
    }

    pub fn empty() -> Self {
        Self {
            external_token: None,
            internal_token: None,
        }
    }
}
