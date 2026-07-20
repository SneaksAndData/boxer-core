mod request_with_claims;

use crate::http::middleware::audit::extract_audit_chain::request_with_claims::RequestWithTokenEvent;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
use actix_web::middleware::Next;
use actix_web::Error;

pub async fn extract_external_token_event<Request: RequestWithTokenEvent>(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let mut request = Request::try_from_request(req).map_err(ErrorInternalServerError)?;
    let event = request.token_event();

    request
        .update_audit_event(|ae| Ok(ae.set_external_token(event)))
        .map_err(ErrorInternalServerError)?;

    next.call(request.into()).await
}
