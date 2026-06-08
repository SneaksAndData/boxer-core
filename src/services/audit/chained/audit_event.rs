use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;

#[derive(Debug, Clone)]
/// [`AuditEvent`] represents the state of the audit information collected during the processing of a request.
pub enum AuditEvent {
    /// [`Final`] indicates that the audit information is complete and should not be modified further.
    Final(ChainedAuditEvent),

    /// [`Intermediate`] indicates that the audit information is still being collected and can be modified.
    Intermediate(ChainedAuditEvent),
}
