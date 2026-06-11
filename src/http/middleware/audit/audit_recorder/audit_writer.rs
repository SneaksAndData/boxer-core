use crate::services::audit::chained::audit_event::AuditEvent;

/// The `AuditWriter` trait defines the contract for writing audit events. It abstracts the logic of
/// persisting or transmitting audit events to the desired destination.
pub trait AuditWriter: Send + Sync + 'static {
    /// Writes the given `AuditEvent` to the configured audit destination.
    fn write(&self, event: AuditEvent);
}
