use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::dev::ServiceRequest;

/// Adds an external token identifier to request-scoped audit context.
///
/// Implementations are responsible for validating that audit context exists in request
/// extensions and that adding the token id does not violate domain invariants
/// (for example, duplicate token identifiers).
pub trait RequestWithTokenId: From<ServiceRequest> {
    /// Token type used to derive the external token identifier.
    type Token: TokenWithId + Clone;

    /// Stores the provided token in the request context and returns the updated request.
    /// The method additionally enriches the audit event coming to the request with the token id.
    /// This method should be called to add the token id and convert the request to the appropriate
    /// type for downstream handlers and middleware.
    fn add_token(self, token: Self::Token) -> ServiceRequest;
}
