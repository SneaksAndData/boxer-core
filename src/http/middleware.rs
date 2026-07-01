use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::HeaderValue;
use actix_web::middleware::Next;
use request_with_token_id::RequestWithTokenId;

pub mod audit;
pub mod extract_external_token;
pub mod extract_internal_token;
pub mod logging;
pub mod request_with_token_id;
pub mod tracer;

async fn extract_token_from_header<TokenType, Request, Error>(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error>
where
    TokenType: TokenWithId + TryFrom<HeaderValue, Error = anyhow::Error> + 'static,
    Error: ExternalTokenError + 'static,
    Request: RequestWithTokenId<Token = TokenType>,
{
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
