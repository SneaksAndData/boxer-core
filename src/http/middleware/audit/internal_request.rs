#[cfg(test)]
mod tests;
pub mod upgrade_version;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::contracts::dynamic_claims_collection::{DynamicClaims, DynamicClaimsCollection};
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::contracts::internal_token::v1::boxer_claims::ToBoxerClaims as V1ToBoxerClaims;
use crate::contracts::internal_token::v2::boxer_claims::ToBoxerClaims as V2ToBoxerClaims;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use crate::http::middleware::request_with_token_id::RequestWithTokenId;
use crate::http::middleware::token_decryptor_middleware::request_with_token::RequestWithToken;
use crate::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpMessage;
use anyhow;
use anyhow::Result;
use upgrade_version::UpgradeVersion;

/// [`InternalRequest`] is a wrapper around `ServiceRequest` that indicates the request has been
/// processed by the `begin_audit_chain` middleware and has an audit context initialized.
/// This struct is used to ensure that the audit context is properly initialized and to prevent
/// multiple initializations of the audit context for the same request.
#[derive(Debug)]
pub struct InternalRequest(ServiceRequest);

impl InternalRequest {
    fn update_audit_event<F>(&mut self, callback: F) -> Result<(), anyhow::Error>
    where
        F: for<'e> FnOnce(&'e mut IntermediateAuditEvent) -> Result<(), anyhow::Error>,
    {
        let mut extensions = self.0.extensions_mut();
        let audit_event = extensions
            .get_mut::<AuditEvent>()
            .ok_or_else(|| anyhow::anyhow!("InternalRequest: Audit event not found in request extensions"))?;

        match audit_event {
            AuditEvent::Final(_) => Err(anyhow::anyhow!(
                "InternalRequest: Audit event already final, cannot modify it"
            )),
            AuditEvent::Intermediate(iae) => callback(iae),
        }
    }
}

/// Implementing `Into<ServiceRequest>` allows us to easily convert an `AuditedRequest` back into
/// a `ServiceRequest` when passing it to the next middleware or handler in the chain.
impl Into<ServiceRequest> for InternalRequest {
    fn into(self) -> ServiceRequest {
        self.0
    }
}

/// The `TryCreateAuditContext` trait is implemented for `AuditedRequest` to define the logic for
/// creating an audit context from a `ServiceRequest`. The implementation checks if the request
/// already contains an `AuditEvent` in its extensions, which would indicate that an audit context
/// has already been initialized.
impl TryCreateAuditContext for InternalRequest {
    fn try_create_audit_context(request: ServiceRequest) -> Result<Self, actix_web::Error> {
        if request.extensions().get::<AuditEvent>().is_some() {
            return Err(ErrorInternalServerError(
                "Failed to create audited request: audit chain already exists in request extensions",
            ));
        }
        request
            .extensions_mut()
            .insert(AuditEvent::Intermediate(IntermediateAuditEvent::empty()));
        Ok(InternalRequest(request))
    }
}

impl AuditEventSource<IntermediateAuditEvent> for InternalRequest {
    type Error = anyhow::Error;

    /// Returns the current [`IntermediateAuditEvent`] stored in the request extensions.
    ///
    fn audit_event(&self) -> Result<IntermediateAuditEvent> {
        let ae = self
            .0
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("AuditEvent not found in InternalRequest extensions"))?;

        match ae {
            AuditEvent::Final(_) => Err(anyhow::anyhow!("Unexpected Final audit event in request extensiosns")),
            AuditEvent::Intermediate(iae) => Ok(iae),
        }
    }
}

impl RequestWithTokenId for InternalRequest {
    type Token = EncryptedToken;

    /// Stores the external token identifier in the request's audit context and returns
    /// the underlying [`ServiceRequest`].
    ///
    /// The token id is derived from the provided [`EncryptedToken`] and written into the [`IntermediateAuditEvent`]
    /// held in request extensions.
    fn add_token(&mut self, token: Self::Token) -> Result<()> {
        let token_id = token.id();

        {
            self.update_audit_event(|e: &mut IntermediateAuditEvent| {
                e.internal_token = Some(TokenAuditEvent::external(token_id));
                Ok(())
            })?;
            let mut binding = self.0.extensions_mut();
            binding.insert(token.clone());
        }
        Ok(())
    }
}

impl RequestWithToken for InternalRequest {
    fn token(&self) -> EncryptedToken {
        self.0
            .extensions()
            .get::<EncryptedToken>()
            .cloned()
            .expect("Encrypted token not exists in request extensions")
    }

    fn set_claims(mut self, claims: DynamicClaimsCollection) -> Result<ServiceRequest, anyhow::Error> {
        // If we have the token version 2, we should extract the audit event from the token and

        let version = claims.get_version()?;
        let boxer_claims = match version.as_str() {
            "v1" => {
                let claims_v1 = V1ToBoxerClaims::to_boxer_claims(&claims)?;
                let event = self.audit_event()?;
                claims_v1.upgrade_version(event.internal_token)
            }
            "v2" => V2ToBoxerClaims::to_boxer_claims(&claims)?,
            _ => return Err(anyhow::anyhow!("Unexpected claims version: {:?}", version)),
        };

        self.update_audit_event(|e: &mut IntermediateAuditEvent| {
            e.external_token = boxer_claims.audit_event.clone();
            Ok(())
        })?;

        self.0.extensions_mut().insert(boxer_claims);
        Ok(self.0)
    }

    fn try_from_request(request: ServiceRequest) -> Result<Self, anyhow::Error> {
        if !request.extensions().contains::<EncryptedToken>() {
            anyhow::bail!("Missing required encrypted token extension");
        };

        if !request.extensions().contains::<AuditEvent>() {
            panic!("Request does not contain AuditEvent in request extensions");
        }

        let internal_request = InternalRequest(request);
        Ok(internal_request)
    }
}

impl TryFrom<ServiceRequest> for InternalRequest {
    type Error = AuditedError;
    fn try_from(value: ServiceRequest) -> Result<Self, Self::Error> {
        if !value.extensions().contains::<AuditEvent>() {
            return Err(AuditedError::from_request(
                &value,
                ErrorInternalServerError("ExternalRequest: Audit event not found in request extensions"),
            ));
        }
        Ok(InternalRequest(value))
    }
}
