pub mod audit_facade;
pub mod authorization_audit_event;
pub mod log_audit_service;
mod resource_delete_audit_event;
mod resource_modification_audit_event;

use crate::services::audit::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::resource_modification_audit_event::ResourceModificationAuditEvent;
use anyhow::Result;

pub trait AuditService: Send + Sync {
    fn record_authorization(&self, event: AuthorizationAuditEvent) -> Result<()>;
    fn record_resource_deletion(&self, event: ResourceDeleteAuditEvent) -> Result<()>;
    fn record_resource_modification(&self, event: ResourceModificationAuditEvent) -> Result<()>;
}
