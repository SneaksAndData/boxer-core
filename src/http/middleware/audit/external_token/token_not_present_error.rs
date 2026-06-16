use crate::http::middleware::audit::external_token::token_with_id::TokenWithId;
use actix_web::dev::ServiceRequest;
use actix_web::ResponseError;

pub trait ExternalTokenError: ResponseError + Send + 'static {
    fn external_token_not_present(request: &ServiceRequest) -> Self;

    fn token_extraction_failed(request: &ServiceRequest, cause: TokenWithId::Error) -> Self;
}
