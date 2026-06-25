use actix_web::ResponseError;
use actix_web::dev::ServiceRequest;

/// Error contract for failures related to extracting the external token from an incoming request.
///
/// Implementors provide constructors for the two middleware failure cases:
/// missing `Authorization` header and token parsing/validation failure.
pub trait ExternalTokenError: ResponseError {
    /// Builds an error for requests that do not include an `Authorization` header.
    fn external_token_not_present(request: &ServiceRequest) -> Self;

    /// Builds an error for requests where the `Authorization` header exists
    /// but the token cannot be extracted or parsed.
    fn token_extraction_failed(request: &ServiceRequest, cause: anyhow::Error) -> Self;
}
