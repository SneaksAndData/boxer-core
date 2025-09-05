pub mod audit_facade;
pub mod events;
pub mod log_audit_service;

use crate::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::events::resource_modification_audit_event::ResourceModificationAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationEvent;
use anyhow::Result;

pub trait AuditService: Send + Sync {
    fn record_authorization(&self, event: AuthorizationAuditEvent) -> Result<()>;
    fn record_resource_deletion(&self, event: ResourceDeleteAuditEvent) -> Result<()>;
    fn record_resource_modification(&self, event: ResourceModificationAuditEvent) -> Result<()>;
    fn record_token_validation(&self, event: TokenValidationEvent) -> Result<()>;
}
