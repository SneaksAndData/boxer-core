use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use maplit::hashset;

#[derive(Debug, Clone)]
/// [`AuditEvent`] represents the state of the audit information collected during the processing of a request.
pub enum AuditEvent {
    /// [`Final`] indicates that the audit information is complete and should not be modified further.
    Final(ChainedAuditEvent),

    /// [`Intermediate`] indicates that the audit information is still being collected and can be modified.
    Intermediate(ChainedAuditEvent),
}

pub struct IntermediateAuditEvent;

impl AuditEvent {
    pub fn token_not_present() -> AuditEvent {
        AuditEvent::Final(ChainedAuditEvent {
            external_token: Some(TokenAuditEvent {
                token_id: None,
                result: Some(TokenValidationResult::Deny),
                reason_errors: hashset! {
                    "token-not-present".into()
                },
                token_type: None,
            }),
            internal_token: None,
            policy_evaluation_result: Some(PolicyEvaluationResult::empty_deny()),
        })
    }

    pub(crate) fn token_extraction_failed(reason: String) -> AuditEvent {
        AuditEvent::Final(ChainedAuditEvent {
            external_token: Some(TokenAuditEvent {
                token_id: None,
                result: Some(TokenValidationResult::Deny),
                reason_errors: hashset! {
                    format!("token-extraction-failed: {}", reason)
                },
                token_type: None,
            }),
            internal_token: None,
            policy_evaluation_result: Some(PolicyEvaluationResult::empty_deny()),
        })
    }

    pub fn finalize(&mut self, result: PolicyEvaluationResult) {
        match self {
            AuditEvent::Intermediate(e) => {
                *self = AuditEvent::Final(ChainedAuditEvent {
                    external_token: e.external_token.clone(),
                    internal_token: e.internal_token.clone(),
                    policy_evaluation_result: Some(result),
                })
            }
            _ => panic!("Cannot finalize an AuditEvent that is already Final"),
        }
    }

    pub fn external_token_data(&self) -> Option<TokenAuditEvent> {
        match self {
            AuditEvent::Final(e) => e.external_token.clone(),
            AuditEvent::Intermediate(e) => e.external_token.clone(),
        }
    }
}
