use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use actix_web::dev::ServiceRequest;

/// Adds an external token identifier to request-scoped audit context.
///
/// Implementations are responsible for validating that audit context exists in request
/// extensions and that adding the token id does not violate domain invariants
/// (for example, duplicate token identifiers).
pub trait RequestWithTokenId: From<ServiceRequest> {
    /// Token type used to derive the external token identifier.
    type Token: TokenWithId;

    /// Stores the provided token id in the request audit context and returns the updated request.
    /// This method should be called to add the token id and convert the request to the appropriate
    /// type for downstream handlers and middleware.
    fn add_external_token_id(self, token: &Self::Token) -> ServiceRequest;
}
