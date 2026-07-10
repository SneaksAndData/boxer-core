use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::authorization_audit_event::Reason;
use cedar_policy::Decision;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// [`ChainedAuditEvent`] represents the information collected during the processing of a
/// request that is relevant for auditing purposes. It includes details about the external and
/// internal token validation, the action being performed, the actor, the resource, the
/// decision made by the authorization engine, and any reasons for that decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedAuditEvent {
    pub external_token: Option<TokenAuditEvent>,
    pub internal_token: Option<TokenAuditEvent>,
    pub policy_evaluation_result: Option<PolicyEvaluationResult>,
}

impl ChainedAuditEvent {
    /// Creates a new empty `ChainedAuditEvent` with all fields set to `None`.
    pub fn empty() -> ChainedAuditEvent {
        ChainedAuditEvent {
            external_token: None,
            internal_token: None,
            policy_evaluation_result: None,
        }
    }

    /// Checks if the `ChainedAuditEvent` is empty
    pub fn is_empty(&self) -> bool {
        self.external_token.is_none() && self.internal_token.is_none() && self.policy_evaluation_result.is_none()
    }

    /// Creates an empty audit event with an external token id
    pub fn external(token_id: &str) -> ChainedAuditEvent {
        ChainedAuditEvent {
            external_token: Some(TokenAuditEvent::external().with_token_id(token_id)),
            internal_token: None,
            policy_evaluation_result: None,
        }
    }

    pub fn action(&self) -> String {
        self.policy_evaluation_result
            .clone()
            .map(|a| a.action)
            .flatten()
            .unwrap_or("unknown".to_string())
    }

    pub fn actor(&self) -> String {
        self.policy_evaluation_result
            .clone()
            .map(|a| a.actor)
            .flatten()
            .unwrap_or("unknown".to_string())
    }

    pub fn resource(&self) -> String {
        self.policy_evaluation_result
            .clone()
            .map(|a| a.resource)
            .flatten()
            .unwrap_or("unknown".to_string())
    }

    pub fn reason(&self) -> Reason {
        self.policy_evaluation_result
            .clone()
            .map(|a| a.reason)
            .flatten()
            .unwrap_or(Reason {
                policies: HashSet::new(),
                errors: HashSet::new(),
            })
    }

    pub fn decision(&self) -> Decision {
        self.policy_evaluation_result
            .clone()
            .map(|a| a.decision)
            .unwrap_or(Decision::Deny)
    }
}
