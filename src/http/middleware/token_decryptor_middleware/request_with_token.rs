use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audited_error::AuditedError;
use actix_web::dev::ServiceRequest;

/// Abstraction over request types that carry an encrypted internal token.
pub trait RequestWithToken: TryFrom<ServiceRequest, Error = AuditedError> + AuditEventSource {
    /// Returns the encrypted token extracted from the request.
    fn token(&self) -> EncryptedToken;

    /// Stores decrypted claims in the request context and returns the updated request.
    fn set_claims(&mut self, claims: DynamicClaimsCollection) -> ServiceRequest;
}
