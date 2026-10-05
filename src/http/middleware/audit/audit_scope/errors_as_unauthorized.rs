use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::middleware::Next;

#[cfg(test)]
mod tests;

/// Changes only propagated errors to HTTP 401, without retaining the request.
pub(super) async fn errors_as_unauthorized<B: MessageBody>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, actix_web::Error> {
    next.call(request).await.map_err(|error| {
        let mut response = error.error_response();
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        InternalError::from_response(error, response).into()
    })
}
