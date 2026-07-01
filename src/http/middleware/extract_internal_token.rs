use super::request_with_token_id::RequestWithTokenId;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;

/// Extracts the internal token from the incoming request and inserts the extracted token to
/// request extensions for further processing. This method does neither decrypt nor validates the
/// extracted token, it inserts a token ID to the AuditEvent for the incoming request if
/// the token is successfully extracted. Additionally, this method inserts the raw token value for
/// further processing. The main purpose os this method is unification the audit logging.
pub async fn extract_encrypted_token<Request, Error>(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error>
where
    Error: ExternalTokenError + 'static,
    Request: RequestWithTokenId<Token = EncryptedToken>,
{
    super::extract_token_from_header::<EncryptedToken, Request, Error>(request, next).await
}
