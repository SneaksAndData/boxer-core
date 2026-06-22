#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::external_token::request_with_token_id::RequestWithTokenId;
use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use crate::models::external_token::ExternalToken;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
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

impl AuditEventSource for AuditedRequest {
    /// Returns the current [`AuditEvent`] stored in the request extensions.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an `AuditEvent` extension.
    /// This should never happen for a properly constructed [`AuditedRequest`],
    /// since `try_create_audit_context` always inserts an event on creation.
    fn audit_event(&self) -> AuditEvent {
        self.0
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions")
    }
}

impl From<ServiceRequest> for AuditedRequest {
    /// Wraps a [`ServiceRequest`] into an [`AuditedRequest`], asserting that an audit context
    /// is already present in request extensions.
    ///
    /// This is the counterpart to [`Into<ServiceRequest>`] and is used by the external token
    /// middleware to re-wrap the request after extracting the token, preserving the existing
    /// audit context.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an [`AuditEvent`] extension.

    fn from(value: ServiceRequest) -> Self {
        value
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions");
        AuditedRequest(value)
    }
}

impl RequestWithTokenId for AuditedRequest {
    type Token = ExternalToken;

    /// Stores the external token identifier in the request's audit context and returns
    /// the underlying [`ServiceRequest`].
    ///
    /// The token id is derived from the provided [`ExternalToken`] and written into the
    /// intermediate [`ChainedAuditEvent`] held in request extensions.
    ///
    /// # Panics
    ///
    /// Panics if the request extensions already contain an external token audit event,
    /// indicating a duplicate token id assignment.
    ///
    /// Panics if the audit event in extensions is not an `AuditEvent::Intermediate`,
    /// which would mean the audit chain is in an unexpected state.
    fn add_token_id(self, token: &Self::Token) -> ServiceRequest {
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
                chained_audit_event.external_token = Some(TokenAuditEvent::external().with_token_id(&token_id))
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
