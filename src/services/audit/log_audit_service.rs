use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::services::audit::chained::audit_event::AuditEvent;
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
    // COVERAGE: disabled since this should be tested in integration tests only
    #[cfg_attr(coverage, coverage(off))]
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

    // COVERAGE: disabled since this should be tested in integration tests only
    #[cfg_attr(coverage, coverage(off))]
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

    // COVERAGE: disabled since this should be tested in integration tests only
    #[cfg_attr(coverage, coverage(off))]
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

    // COVERAGE: disabled since this should be tested in integration tests only
    #[cfg_attr(coverage, coverage(off))]
    fn record_token_validation(&self, event: TokenValidationEvent) -> Result<()> {
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            id = event.token_id.as_str(),
            result:serde = event.result,
            token_type = event.token_type.as_str(),
            reason_errors:serde = event.reason_errors,
            metadata:serde = event.token_metadata;

            // The log message
            "Boxer token validation: {:?}/{:?}", event.token_type, event.token_id);

        Ok(())
    }
}

impl AuditWriter for LogAuditService {
    /// Writes an audit event as a structured log entry.
    ///
    /// Both intermediate and final events are normalized into a single payload
    /// shape and emitted with `log_type = "audit"` so downstream log systems can
    /// reliably filter and index audit records.

    // COVERAGE: disabled since this should be tested in integration tests only
    #[cfg_attr(coverage, coverage(off))]
    fn write(&self, event: AuditEvent) {
        let (payload, is_final) = match event {
            AuditEvent::Final(e) => (e, true),
            AuditEvent::Intermediate(e) => (e, false),
        };

        let decision = payload.decision();
        let reason = payload.reason();
        let external_token_id = payload.external_token.as_ref().and_then(|t| t.token_id.clone());
        let internal_token_id = payload.internal_token.as_ref().and_then(|t| t.token_id.clone());
        log::info!(
            // Indicates the audit events for easier filtering in log aggregation systems
            log_type = "audit",

            // The event decomposition for structured logging
            is_final = is_final,
            action = payload.action(),
            actor = payload.actor(),
            resource = payload.resource(),
            decision:serde = decision,
            reason_policies:serde = reason.policies,
            reason_errors:serde = reason.errors,
            external_token_id = external_token_id,
            internal_token_id = internal_token_id;

            // The log message
            "Boxer audit event recorded with decision: {:?}", decision
        );
    }
}
