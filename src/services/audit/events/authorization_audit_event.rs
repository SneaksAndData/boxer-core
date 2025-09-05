use cedar_policy::{Decision, Diagnostics, EntityUid, Response};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct AuthorizationAuditEvent {
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub decision: Decision,
    pub reason: Reason,
}

#[derive(Serialize)]
pub struct Reason {
    pub policies: HashSet<String>,
    pub errors: HashSet<String>,
}

impl From<&Diagnostics> for Reason {
    fn from(diagnostics: &Diagnostics) -> Self {
        Self {
            policies: diagnostics.reason().map(|p| p.to_string().clone()).collect(),
            errors: diagnostics.errors().map(|e| e.to_string().clone()).collect(),
        }
    }
}

impl AuthorizationAuditEvent {
    pub fn new(actor: &EntityUid, action: &EntityUid, resource: &EntityUid, response: &Response) -> Self {
        Self {
            action: action.to_string(),
            actor: actor.to_string(),
            resource: resource.to_string(),
            decision: response.decision(),
            reason: Reason::from(response.diagnostics()),
        }
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }
}
