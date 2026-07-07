use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audited_error::AuditedError;
use actix_web::dev::ServiceRequest;

/// Abstraction over request types that carry an encrypted internal token.
pub trait RequestWithToken: AuditEventSource {
    /// Returns the encrypted token extracted from the request.
    fn token(&self) -> EncryptedToken;

    /// Extracts encrypted token from ServiceRequest.
    fn try_from_request(request: ServiceRequest) -> Result<Self, AuditedError>
    where
        Self: Sized;

    /// Stores decrypted claims in the request context and returns the updated request.
    fn set_claims(self, claims: DynamicClaimsCollection) -> Result<ServiceRequest, anyhow::Error>;
}
