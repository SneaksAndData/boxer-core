pub mod external_token_error;
pub mod request_with_token_id;
pub mod token_with_id;

use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use request_with_token_id::RequestWithTokenId;

/// Extracts the external token from the incoming request and inserts the extracted token to
/// request extensions for further processing. If the token is not present or extraction fails,
/// returns an error response. Inserts a token ID to the AuditEvent for the incoming request if
/// the token is successfully extracted.
pub async fn extract_external_token<Request: RequestWithTokenId, Error: ExternalTokenError + 'static>(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let header_value = request.headers().get("Authorization").cloned();

    match header_value {
        None => Err(Error::external_token_not_present(&request).into()),

        Some(header_value) => {
            let token =
                Request::Token::try_from(header_value).map_err(|e| Error::token_extraction_failed(&request, e))?;
            next.call(Request::from(request).add_token(token)).await
        }
    }
}
