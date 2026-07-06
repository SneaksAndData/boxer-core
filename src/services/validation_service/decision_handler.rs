use crate::services::audit::chained::audit_event::AuditEvent;
use cedar_policy::{EntityUid, Response};

pub trait DecisionHandler: Send + Sync {
    fn handle(&self, x: &EntityUid, x0: &EntityUid, x1: &EntityUid, x2: &Response) -> Option<AuditEvent>;
}
