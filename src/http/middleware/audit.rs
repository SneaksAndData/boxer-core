use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpMessage, HttpResponse};
use maplit::hashset;
use status_filter::StatusFilter;

pub mod audit_recorder;
pub mod audited_error;
pub mod audited_response;
pub mod begin_audit_chain;
pub mod enrich_from_origin;
pub mod external_request;
pub mod internal_request;
pub mod origin_context;
pub mod status_filter;

pub mod audit_scope;
pub mod extract_external_token_event;
#[cfg(test)]
mod tests;

/// Finalizes the audit event when [`StatusFilter::should_finalize`] returns `true`.
///
/// Register with `from_fn(finalize_on_status::<YourStatusFilter>)`.
/// Responses that are not skipped require an [`AuditEvent`] in request extensions.
/// Service errors are propagated unchanged.
pub async fn finalize_on_status<F: StatusFilter>(
    request: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let response = next.call(request.into()).await;

    match response {
        Ok(response) if F::should_finalize(response.status()) => {
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
                event.try_finalize(result);
            }

            let mut builder = HttpResponse::build(status);
            for (k, v) in &headers {
                builder.insert_header((k.clone(), v.clone()));
            }

            let restored = builder.body(body_bytes).map_into_boxed_body();
            Ok(ServiceResponse::new(req, restored))
        }
        Ok(response) => Ok(response.map_into_boxed_body()),
        Err(error) => Err(error),
    }
}
