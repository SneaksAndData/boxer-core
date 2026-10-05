use crate::http::middleware::audit::finalize_on_status;
use crate::http::middleware::audit::status_filter::skip_sucessfull_statuses::SkipSuccessfulStatuses;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;

/// Finalizes the audit event for non-successful responses, leaving 2xx responses unchanged.
pub async fn finalize_on_fail(
    request: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    finalize_on_status::<SkipSuccessfulStatuses>(request, next).await
}
