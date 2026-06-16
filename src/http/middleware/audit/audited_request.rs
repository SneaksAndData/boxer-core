#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use actix_web::HttpMessage;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;

/// [`AuditedRequest`] is a wrapper around `ServiceRequest` that indicates the request has been
/// processed by the `begin_audit_chain` middleware and has an audit context initialized.
/// This struct is used to ensure that the audit context is properly initialized and to prevent
/// multiple initializations of the audit context for the same request.
#[derive(Debug)]
pub struct AuditedRequest(ServiceRequest);

/// Implementing `Into<ServiceRequest>` allows us to easily convert an `AuditedRequest` back into
/// a `ServiceRequest` when passing it to the next middleware or handler in the chain.
impl Into<ServiceRequest> for AuditedRequest {
    fn into(self) -> ServiceRequest {
        self.0
    }
}

/// The `TryCreateAuditContext` trait is implemented for `AuditedRequest` to define the logic for
/// creating an audit context from a `ServiceRequest`. The implementation checks if the request
/// already contains an `AuditEvent` in its extensions, which would indicate that an audit context
/// has already been initialized.
impl TryCreateAuditContext for AuditedRequest {
    fn try_create_audit_context(request: ServiceRequest) -> Result<Self, actix_web::Error> {
        if request.extensions().get::<AuditEvent>().is_some() {
            return Err(ErrorInternalServerError(
                "Failed to create audited request: audit chain already exists in request extensions",
            ));
        }
        request
            .extensions_mut()
            .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));
        Ok(AuditedRequest(request))
    }
}