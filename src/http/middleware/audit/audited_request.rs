#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::external_token::request_with_token_id::RequestWithTokenId;
use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use crate::models::external_token::ExternalToken;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
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

impl AuditEventSource for AuditedRequest {
    fn audit_event(&self) -> AuditEvent {
        self.0
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions")
    }
}

impl From<ServiceRequest> for AuditedRequest {
    fn from(value: ServiceRequest) -> Self {
        value
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions");
        AuditedRequest(value)
    }
}

impl<'a> TryFrom<&'a ServiceResponse> for AuditedRequest {
    type Error = actix_web::Error;

    fn try_from(_value: &'a ServiceResponse) -> Result<AuditedRequest, Self::Error> {
        todo!()
    }
}

impl RequestWithTokenId for AuditedRequest {
    type Token = ExternalToken;

    fn add_external_token_id(self, token: &Self::Token) -> ServiceRequest {
        let token_id = token.id();

        {
            let mut binding = self.0.extensions_mut();
            let audit_event = binding.get_mut::<AuditEvent>();

            // Mutate the audit event if the audit event complains the expected structure
            if let Some(AuditEvent::Intermediate(chained_audit_event)) = audit_event {
                if chained_audit_event.external_token.is_some() {
                    panic!(
                        "External token audit event already exists in request extensions: {:?}",
                        chained_audit_event.external_token
                    );
                }
                chained_audit_event
                    .external_token
                    .as_mut()
                    .expect("External token audit event not exists in request extensions")
                    .token_id = Some(token_id);
            } else {
                // Otherwise, stop processing immediately
                panic!(
                    "Expected Intermediate Audit event to exist in request extension, but got {:?}",
                    audit_event
                );
            }
        }

        // Return the updated value
        self.0
    }
}
