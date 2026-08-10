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
    pub fn get_properties(&self) -> AuditEventProperties {
        let mut properties = AuditEventProperties {
            is_final: true,
            decision: self.policy_evaluation_result.decision.clone(),
            ..Default::default()
        };

        properties.action = self.policy_evaluation_result.action.clone().unwrap_or_default();
        properties.actor = self.policy_evaluation_result.actor.clone().unwrap_or_default();
        properties.resource = self.policy_evaluation_result.resource.clone().unwrap_or_default();
        properties.reason = self.policy_evaluation_result.reason.clone().unwrap_or_else(|| {
            crate::services::audit::events::authorization_audit_event::Reason {
                policies: Default::default(),
                errors: Default::default(),
            }
        });
        properties.external_token_id = self
            .external_token
            .as_ref()
            .map(|token| token.token_id.clone())
            .unwrap_or_default();
        properties.internal_token_id = self
            .internal_token
            .as_ref()
            .map(|token| token.token_id.clone())
            .unwrap_or_default();

        properties
    }
}

impl FinalAuditEvent {
    pub fn token_not_present() -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: String::default(),
                reason_errors: hashset! {
                    "token-not-present".into()
                },
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::empty_deny(),
        }
    }

    pub(crate) fn external_token_extraction_failed(reason: String) -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: String::default(),
                reason_errors: hashset! {
                    format!("token-extraction-failed: {}", reason)
                },
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::empty_deny(),
        }
    }

    pub(crate) fn internal_token_extraction_failed(reason: String) -> Self {
        Self {
            external_token: None,
            internal_token: Some(TokenAuditEvent {
                token_id: String::default(),
                reason_errors: hashset! {
                    format!("token-extraction-failed: {}", reason)
                },
            }),
            policy_evaluation_result: PolicyEvaluationResult::empty_deny(),
        }
    }
}
