pub mod audit_event_properties;
pub mod final_audit_event;
pub mod intermediate_audit_event;

use crate::services::audit::chained::audit_event::audit_event_properties::AuditEventProperties;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use final_audit_event::FinalAuditEvent;
use intermediate_audit_event::IntermediateAuditEvent;

#[derive(Debug, Clone)]
/// [`AuditEvent`] represents the state of the audit information collected during the processing of a request.
pub enum AuditEvent {
    /// [`Final`] indicates that the audit information is complete and should not be modified further.
    Final(FinalAuditEvent),

    /// [`Intermediate`] indicates that the audit information is still being collected and can be modified.
    Intermediate(IntermediateAuditEvent),
}

impl AuditEvent {
    pub(crate) fn finalize_internal_token_error(&self, cause: String) -> AuditEvent {
        match self {
            AuditEvent::Intermediate(event) => {
                let final_event = event.finalize_internal_token_error(cause);
                AuditEvent::Final(final_event)
            }
            AuditEvent::Final(event) => panic!("AuditEvent is already finalized: {:?}", event),
        }
    }
}

impl AuditEvent {
    pub fn get_properties(self) -> AuditEventProperties {
        match self {
            AuditEvent::Final(event) => event.get_properties(),
            AuditEvent::Intermediate(event) => event.get_properties(),
        }
    }

    pub fn finalize(&mut self, result: PolicyEvaluationResult) -> () {
        match self {
            AuditEvent::Intermediate(event) => {
                let final_event = event.finalize(result);
                *self = AuditEvent::Final(final_event.clone());
            }
            AuditEvent::Final(event) => panic!("AuditEvent is already finalized: {:?}", event),
        }
    }

    pub fn set_external_token(&mut self, external_token: TokenAuditEvent) {
        match self {
            AuditEvent::Intermediate(e) => {
                *self = AuditEvent::Intermediate(IntermediateAuditEvent {
                    external_token: Some(external_token),
                    internal_token: e.internal_token.clone(),
                })
            }
            _ => panic!("Cannot finalize an AuditEvent that is already Final"),
        }
    }
}
