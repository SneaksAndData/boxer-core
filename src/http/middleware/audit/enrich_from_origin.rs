pub mod enrich_from_origin;
#[cfg(test)]
mod tests;

pub use super::origin_context::OriginContext;
use actix_web::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use enrich_from_origin::EnrichFromOrigin;

/// Enriches final audit events before they reach the audit recorder on the response path.
///
/// Register inside `AuditRecorderFactory`, outside middleware that creates or finalizes events.
pub async fn enrich_from_origin<ResponseContext, ErrorContext, Body>(
    request: ServiceRequest,
    next: Next<Body>,
) -> Result<ServiceResponse<Body>, Error>
where
    Body: MessageBody,
    ResponseContext: EnrichFromOrigin<ServiceResponse<Body>>,
    ErrorContext: EnrichFromOrigin<Error>,
{
    let context = OriginContext::from_request(&request);
    match next.call(request).await {
        Ok(response) => Ok(ResponseContext::enrich_from_origin(response, context)),
        Err(error) => Err(ErrorContext::enrich_from_origin(error, context)),
    }
}
