#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use crate::http::middleware::audit::external_token::with_external_token_id::WithExternalTokenId;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::HeaderValue;
use actix_web::HttpMessage;

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

impl From<ServiceRequest> for AuditedRequest {
    fn from(value: ServiceRequest) -> Self {
        todo!()
    }
}

impl WithExternalTokenId for AuditedRequest {
    type Token = String;

    fn with_external_token_id(self, token: &Self::Token) -> ServiceRequest {
        todo!()
    }
}

impl TokenWithId for String {
    fn id() -> String {
        todo!()
    }
}

impl TryFrom<&HeaderValue> for String {
    type Error = ();

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        todo!()
    }
}
