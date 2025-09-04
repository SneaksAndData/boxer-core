use crate::services::audit::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::resource_modification_audit_event::ResourceModificationAuditEvent;
use crate::services::audit::AuditService;
use anyhow::Result;

pub struct LogAuditService;

impl LogAuditService {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuditService for LogAuditService {
    fn record_authorization(&self, event: AuthorizationAuditEvent) -> Result<()> {
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            action = event.action.as_str(),
            actor = event.actor.as_str(),
            resource = event.resource.as_str(),
            decision:serde = event.decision,
            reason_policies:serde = event.reason.policies,
            reason_errors:serde = event.reason.errors;

            // The log message
            "Authorization to access the resource: {:?}", event.resource());

        Ok(())
    }

    fn record_resource_deletion(&self, event: ResourceDeleteAuditEvent) -> Result<()> {
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.id.as_str(),
            resource_type = event.resource_type.as_str(),
            successfull = event.successful;

            // The log message
            "Boxer resource deleted: {:?}/{:?}", event.resource_type, event.id);

        Ok(())
    }

    fn record_resource_modification(&self, event: ResourceModificationAuditEvent) -> Result<()> {
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.id.as_str(),
            resource_type = event.resource_type.as_str(),
            successfull:serde = event.modification_result;

            // The log message
            "Boxer resource modified: {:?}/{:?}", event.resource_type, event.id);

        Ok(())
    }
}
