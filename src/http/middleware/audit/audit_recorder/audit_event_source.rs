use crate::services::audit::chained::audit_event::AuditEvent;

/// [`AuditEventSource`] is a trait that defines a source of audit events.
/// It provides a method to retrieve the current audit event, which can be used by
/// the [`AuditRecorder`] middleware to record audit information for incoming requests.
pub trait AuditEventSource {
    /// Retrieves the current audit event.
    fn audit_event(&self) -> AuditEvent;
}
