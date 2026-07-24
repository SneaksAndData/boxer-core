#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use crate::http::middleware::request_with_token_id::RequestWithTokenId;
use crate::models::external_token::ExternalToken;
use crate::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpMessage;

/// [`ExternalRequest`] is a wrapper around `ServiceRequest` that indicates the request has been
/// processed by the `begin_audit_chain` middleware and has an audit context initialized.
/// This struct is used to ensure that the audit context is properly initialized and to prevent
/// multiple initializations of the audit context for the same request.
#[derive(Debug)]
pub struct ExternalRequest(ServiceRequest);

impl ExternalRequest {
    fn update_audit_event<F>(&mut self, callback: F) -> anyhow::Result<(), anyhow::Error>
    where
        F: for<'e> FnOnce(&'e mut IntermediateAuditEvent) -> Result<(), anyhow::Error>,
    {
        let mut extensions = self.0.extensions_mut();
        let audit_event = extensions
            .get_mut::<AuditEvent>()
            .ok_or_else(|| anyhow::anyhow!("ExternalRequest: Audit event not found in request extensions"))?;

        match audit_event {
            AuditEvent::Final(_) => Err(anyhow::anyhow!(
                "ExternalRequest: Audit event already final, cannot modify it"
            )),
            AuditEvent::Intermediate(iae) => callback(iae),
        }
    }
}

/// Implementing `Into<ServiceRequest>` allows us to easily convert an `AuditedRequest` back into
/// a `ServiceRequest` when passing it to the next middleware or handler in the chain.
impl Into<ServiceRequest> for ExternalRequest {
    fn into(self) -> ServiceRequest {
        self.0
    }
}

/// The `TryCreateAuditContext` trait is implemented for `AuditedRequest` to define the logic for
/// creating an audit context from a `ServiceRequest`. The implementation checks if the request
/// already contains an `AuditEvent` in its extensions, which would indicate that an audit context
/// has already been initialized.
impl TryCreateAuditContext for ExternalRequest {
    fn try_create_audit_context(request: ServiceRequest) -> Result<Self, actix_web::Error> {
        if request.extensions().get::<AuditEvent>().is_some() {
            return Err(ErrorInternalServerError(
                "Failed to create audited request: audit chain already exists in request extensions",
            ));
        }
        request
            .extensions_mut()
            .insert(AuditEvent::Intermediate(IntermediateAuditEvent::empty()));
        Ok(ExternalRequest(request))
    }
}

impl AuditEventSource<IntermediateAuditEvent> for ExternalRequest {
    type Error = anyhow::Error;

    /// Returns the current [`IntermediateAuditEvent`] stored in the request extensions.
    fn audit_event(&self) -> Result<IntermediateAuditEvent, Self::Error> {
        self.0
            .extensions()
            .get::<IntermediateAuditEvent>()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Audited event not exists in request extensions"))
    }
}

impl TryFrom<ServiceRequest> for ExternalRequest {
    type Error = AuditedError;

    /// Wraps a [`ServiceRequest`] into an [`ExternalRequest`], asserting that an audit context
    /// is already present in request extensions.
    ///
    /// This is the counterpart to [`Into<ServiceRequest>`] and is used by the external token
    /// middleware to re-wrap the request after extracting the token, preserving the existing
    /// audit context.

    fn try_from(value: ServiceRequest) -> Result<Self, Self::Error> {
        let event = value.extensions().get::<AuditEvent>().cloned().ok_or_else(|| {
            AuditedError::from_request(
                &value,
                ErrorInternalServerError("IntermediateAuditEvent event not exists in request extensions"),
            )
        })?;
        match event {
            AuditEvent::Final(_) => Err(AuditedError::from_request(
                &value,
                ErrorInternalServerError("IntermediateAuditEvent event is already final"),
            )),
            AuditEvent::Intermediate(_) => Ok(ExternalRequest(value)),
        }
    }
}

impl RequestWithTokenId for ExternalRequest {
    type Token = ExternalToken;

    /// Stores the external token identifier in the request's audit context and returns
    /// the underlying [`ServiceRequest`].
    ///
    /// The token id is derived from the provided [`ExternalToken`] and written into the
    /// intermediate [`ChainedAuditEvent`] held in request extensions.
    fn add_token(&mut self, token: Self::Token) -> anyhow::Result<()> {
        let token_id = token.id();

        {
            self.update_audit_event(|e: &mut IntermediateAuditEvent| {
                e.external_token = Some(TokenAuditEvent::external(token_id));
                Ok(())
            })?;
            let mut binding = self.0.extensions_mut();
            binding.insert(token.clone());
        }
        Ok(())
    }
}
