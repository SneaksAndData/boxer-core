use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use actix_web::dev::ServiceRequest;

pub trait RequestWithTokenEvent: Into<ServiceRequest> {
    fn token_event(&self) -> TokenAuditEvent;

    /// Extracts encrypted token from ServiceRequest.
    fn try_from_request(request: ServiceRequest) -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn update_audit_event<F>(&mut self, callback: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(&mut AuditEvent) -> Result<(), anyhow::Error>;
}
