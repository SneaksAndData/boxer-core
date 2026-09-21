use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpMessage, HttpResponse};
use maplit::hashset;

pub mod audit_recorder;
pub mod audited_error;
pub mod audited_response;
pub mod begin_audit_chain;
pub mod external_request;
pub mod internal_request;

pub mod audit_scope;
pub mod extract_external_token_event;
#[cfg(test)]
mod tests;

/// Extracts the external token from the incoming request and inserts the extracted token to
/// request extensions for further processing. If the token is not present or extraction fails,
/// returns an error response. Inserts a token ID to the AuditEvent for the incoming request if
/// the token is successfully extracted.
pub async fn finalize_on_fail(
    request: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let response = next.call(request.into()).await;

    match response {
        Ok(response) if !response.status().is_success() => {
            let (req, res) = response.into_parts();

            let headers = res.headers().clone();
            let status = res.status();
            let body_bytes = res.into_body().try_into_bytes().unwrap_or_default();
            let preview = String::from_utf8_lossy(&body_bytes).into_owned();

            {
                let mut extensions_mut = req.extensions_mut();
                let event = extensions_mut.get_mut::<AuditEvent>().unwrap();

                let result = PolicyEvaluationResult::with_custom_errors(hashset! {
                    format!("Boxer produced response with status status: {}: {}", status, preview)
                });
                event.finalize(result);
            }

            let mut builder = HttpResponse::build(status);
            for (k, v) in &headers {
                builder.insert_header((k.clone(), v.clone()));
            }

            let restored = builder.body(body_bytes).map_into_boxed_body();
            Ok(ServiceResponse::new(req, restored))
        }
        Ok(response) => Ok(response.map_into_boxed_body()),
        Err(_error) => todo!(),
    }
}
