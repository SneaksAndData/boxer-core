use crate::services::audit::chained::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;

pub struct IntermediateAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,
}
impl IntermediateAuditEvent {
    pub fn finalize(&mut self, result: PolicyEvaluationResult) -> FinalAuditEvent {
        FinalAuditEvent {
            external_token: self.external_token.take(),
            internal_token: self.internal_token.take(),
            policy_evaluation_result: result,
        }
    }
}
