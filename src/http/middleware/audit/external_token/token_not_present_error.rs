use actix_web::ResponseError;
use actix_web::dev::ServiceRequest;

pub trait ExternalTokenError: ResponseError {
    fn external_token_not_present(request: &ServiceRequest) -> Self;

    fn token_extraction_failed(request: &ServiceRequest, cause: anyhow::Error) -> Self;
}
