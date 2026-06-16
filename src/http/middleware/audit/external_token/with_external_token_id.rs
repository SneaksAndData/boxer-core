use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use actix_web::dev::ServiceRequest;

/// `AuditExternalToken` is a trait that defines a method for adding an `ExternalToken` to
/// an audit context. The implementation of the trait should validate that the request contains a
/// valid audit event in the request extensions and the external token id is not duplicates.
pub trait WithExternalTokenId: From<ServiceRequest> {
    /// Adds the given `ExternalToken` to the audit context and returns the modified context.
    type Token: TokenWithId;

    fn with_external_token_id(self, token: &Self::Token) -> ServiceRequest;
}
