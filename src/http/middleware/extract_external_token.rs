pub mod external_token_error;
pub mod token_with_id;

use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use crate::http::middleware::request_with_token_id::RequestWithTokenId;
use crate::models::external_token::ExternalToken;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;

/// Extracts the external token from the incoming request and inserts the extracted token to
/// request extensions for further processing. If the token is not present or extraction fails,
/// returns an error response. Inserts a token ID to the AuditEvent for the incoming request if
/// the token is successfully extracted.
pub async fn extract_external_token<Request, Error>(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error>
where
    Error: ExternalTokenError + 'static,
    Request: RequestWithTokenId<Token = ExternalToken>,
{
    super::extract_token_from_header::<ExternalToken, Request, Error>(request, next).await
}
