use crate::services::audit::chained::audit_event::audit_event_properties::AuditEventProperties;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use maplit::hashset;

#[derive(Clone, Debug)]
pub struct FinalAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,
    pub policy_evaluation_result: PolicyEvaluationResult,
}

impl FinalAuditEvent {
    pub fn get_properties(self) -> AuditEventProperties {
        let mut properties = AuditEventProperties::new(self.policy_evaluation_result.decision);

        properties.action = self.policy_evaluation_result.action.unwrap_or_default();
        properties.actor = self.policy_evaluation_result.actor.unwrap_or_default();
        properties.resource = self.policy_evaluation_result.resource.unwrap_or_default();
        properties.reason = self.policy_evaluation_result.reason.unwrap_or_else(|| {
            crate::services::audit::events::authorization_audit_event::Reason {
                policies: Default::default(),
                errors: Default::default(),
            }
        });
        properties.external_token_id = self.external_token.map(|token| token.token_id).unwrap_or_default();
        properties.internal_token_id = self.internal_token.map(|token| token.token_id).unwrap_or_default();

        properties
    }
}

impl FinalAuditEvent {
    pub(crate) fn external_token_extraction_failed(reason: String) -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: String::default(),
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::with_custom_errors(hashset! {
                reason
            }),
        }
    }

    pub(crate) fn internal_token_extraction_failed(reason: String) -> Self {
        Self {
            external_token: None,
            internal_token: Some(TokenAuditEvent {
                token_id: String::default(),
            }),
            policy_evaluation_result: PolicyEvaluationResult::with_custom_errors(hashset! {
                reason
            }),
        }
    }

    #[cfg(test)]
    /// Constructor that should be used in tests. Should not be used in production code.
    pub fn for_test() -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: String::default(),
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::with_custom_errors(hashset! {}),
        }
    }
}
