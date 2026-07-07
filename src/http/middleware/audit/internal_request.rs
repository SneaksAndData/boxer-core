#[cfg(test)]
mod tests;

use super::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::contracts::dynamic_claims_collection::{DynamicClaims, DynamicClaimsCollection};
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::contracts::internal_token::v1::boxer_claims::ToBoxerClaims as V1ToBoxerClaims;
use crate::contracts::internal_token::v2::boxer_claims::ToBoxerClaims as V2ToBoxerClaims;
use crate::contracts::upgrade_version::UpgradeVersion;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use crate::http::middleware::request_with_token_id::RequestWithTokenId;
use crate::http::middleware::token_decryptor_middleware::request_with_token::RequestWithToken;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpMessage;

/// [`InternalRequest`] is a wrapper around `ServiceRequest` that indicates the request has been
/// processed by the `begin_audit_chain` middleware and has an audit context initialized.
/// This struct is used to ensure that the audit context is properly initialized and to prevent
/// multiple initializations of the audit context for the same request.
#[derive(Debug)]
pub struct InternalRequest(ServiceRequest);

impl InternalRequest {
    fn update_audit_event<F>(&mut self, callback: F) -> Result<(), anyhow::Error>
    where
        F: FnOnce(&mut AuditEvent) -> Result<(), anyhow::Error>,
    {
        let mut extensions = self.0.extensions_mut();
        let audit_event = extensions
            .get_mut()
            .ok_or_else(|| anyhow::anyhow!("Audit event not found in request extensions"))?;
        callback(audit_event)
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
            .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));
        Ok(InternalRequest(request))
    }
}

impl AuditEventSource for InternalRequest {
    /// Returns the current [`AuditEvent`] stored in the request extensions.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an `AuditEvent` extension.
    /// This should never happen for a properly constructed [`InternalRequest`],
    /// since `try_create_audit_context` always inserts an event on creation.
    fn audit_event(&self) -> AuditEvent {
        self.0
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions")
    }
}

impl RequestWithTokenId for InternalRequest {
    type Token = EncryptedToken;

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
    fn add_token(self, token: Self::Token) -> ServiceRequest {
        let token_id = token.id();

        {
            let mut binding = self.0.extensions_mut();
            let audit_event = binding.get_mut::<AuditEvent>();

            // Mutate the audit event if the audit event complains the expected structure
            if let Some(AuditEvent::Intermediate(chained_audit_event)) = audit_event {
                if chained_audit_event.internal_token.is_some() {
                    panic!(
                        "External token audit event already exists in request extensions: {:?}",
                        chained_audit_event.internal_token
                    );
                }
                chained_audit_event.internal_token = Some(TokenAuditEvent::external().with_token_id(&token_id))
            } else {
                // Otherwise, stop processing immediately
                panic!(
                    "Expected Intermediate Audit event to exist in request extension, but got {:?}",
                    audit_event
                );
            }

            binding.insert(token.clone());
        }

        // Return the updated value
        self.0
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

    fn try_from_request(request: ServiceRequest) -> Result<Self, AuditedError> {
        if !request.extensions().contains::<EncryptedToken>() {
            return Err(AuditedError::token_not_present(&request));
        };

        if request.extensions().contains::<AuditEvent>() {
            panic!(
                "Audited audit event duplicated in request extensions: {:?}",
                request.extensions().get::<AuditEvent>()
            );
        }

        let internal_request = InternalRequest(request);
        let token = internal_request.token();

        internal_request
            .0
            .extensions_mut()
            .insert(AuditEvent::Intermediate(ChainedAuditEvent::external(&token.id())));

        Ok(internal_request)
    }

    fn set_claims(mut self, claims: DynamicClaimsCollection) -> Result<ServiceRequest, anyhow::Error> {
        // If we have the token version 2, we should extract the audit event from the token and

        let version = claims.get_version()?;
        let boxer_claims = match version.as_str() {
            "v1" => {
                let claims_v1 = V1ToBoxerClaims::to_boxer_claims(&claims)?;
                let event = self.audit_event();
                match event {
                    AuditEvent::Intermediate(e) => claims_v1.upgrade_version(e),
                    _ => anyhow::bail!("Unexpected audit event type when upgrading claims: {:?}", event),
                }
            }
            "v2" => V2ToBoxerClaims::to_boxer_claims(&claims)?,
            _ => return Err(anyhow::anyhow!("Unexpected claims version: {:?}", version)),
        };

        self.update_audit_event(|audit_event| {
            match audit_event {
                AuditEvent::Intermediate(ChainedAuditEvent {
                    external_token: token, ..
                }) => {
                    *token = boxer_claims.audit_event.external_token.clone();
                }
                _ => anyhow::bail!("Unexpected audit event type when setting claims: {:?}", audit_event),
            }
            *audit_event = AuditEvent::Intermediate(boxer_claims.audit_event);
            Ok(())
        })?;

        Ok(self.0)
    }
}

impl TryFrom<ServiceRequest> for InternalRequest {
    type Error = AuditedError;
    fn try_from(value: ServiceRequest) -> Result<Self, Self::Error> {
        todo!()
    }
}
