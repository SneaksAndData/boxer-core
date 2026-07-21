use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use maplit::hashset;

pub struct FinalAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,
    pub policy_evaluation_result: PolicyEvaluationResult,
}

impl FinalAuditEvent {
    pub fn token_not_present() -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: None,
                result: Some(TokenValidationResult::Deny),
                reason_errors: hashset! {
                    "token-not-present".into()
                },
                token_type: None,
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::empty_deny(),
        }
    }

    pub(crate) fn token_extraction_failed(reason: String) -> Self {
        Self {
            external_token: Some(TokenAuditEvent {
                token_id: None,
                result: Some(TokenValidationResult::Deny),
                reason_errors: hashset! {
                    format!("token-extraction-failed: {}", reason)
                },
                token_type: None,
            }),
            internal_token: None,
            policy_evaluation_result: PolicyEvaluationResult::empty_deny(),
        }
    }
}
