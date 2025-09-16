use crate::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::events::resource_modification_audit_event::{
    ModificationResult, ResourceModificationAuditEvent,
};
use crate::services::audit::events::token_validation_event::TokenValidationEvent;
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
            result:serde = event.decision,
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
        if let ModificationResult::Success(result) = &event.modification_result {
            log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.id.as_str(),
            resource_type = event.resource_type.as_str(),
            successfull = result;

            // The log message
            "Boxer resource modified: {:?}/{:?}", event.resource_type, event.id);
        } else {
            log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.id.as_str(),
            resource_type = event.resource_type.as_str(),
            failure:serde = event.modification_result;

            // The log message
            "Boxer resource modified: {:?}/{:?}", event.resource_type, event.id);
        }

        Ok(())
    }

    fn record_token_validation(&self, event: TokenValidationEvent) -> Result<()> {
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.token_id.as_str(),
            result:debug = event.result,
            token_type = event.token_type.as_str();

            // The log message
            "Boxer token validation: {:?}/{:?}", event.token_type, event.token_id);

        Ok(())
    }
}
