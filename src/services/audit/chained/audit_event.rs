use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use cedar_policy::Decision;
use maplit::hashset;

#[derive(Debug, Clone)]
/// [`AuditEvent`] represents the state of the audit information collected during the processing of a request.
pub enum AuditEvent {
    /// [`Final`] indicates that the audit information is complete and should not be modified further.
    Final(ChainedAuditEvent),

    /// [`Intermediate`] indicates that the audit information is still being collected and can be modified.
    Intermediate(ChainedAuditEvent),
}

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
            action: None,
            actor: None,
            resource: None,
            decision: Some(Decision::Deny),
            reason: None,
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
            action: None,
            actor: None,
            resource: None,
            decision: Some(Decision::Deny),
            reason: None,
        })
    }
}
