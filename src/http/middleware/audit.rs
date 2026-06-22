pub mod audit_recorder;
pub mod audited_error;
pub mod audited_request;
pub mod audited_response;
pub mod begin_audit_chain;
pub mod external_token;
#[cfg(test)]
mod tests;

use crate::http::middleware::audit::audit_recorder::audit_recorder_factory::AuditRecorderFactory;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::audited_request::AuditedRequest;
use crate::http::middleware::audit::audited_response::AuditedResponse;
use crate::http::middleware::audit::begin_audit_chain::begin_audit_chain;
use crate::http::middleware::audit::external_token::external_token;
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use actix_web::web;
use std::sync::Arc;

pub fn initial_audit_scope<S: HttpServiceFactory + 'static>(
    target: S,
    audit_writer: Arc<dyn AuditWriter>,
) -> impl HttpServiceFactory {
    web::scope("")
        .wrap(from_fn(external_token::<AuditedRequest, AuditedError>))
        .wrap(AuditRecorderFactory::<AuditedResponse<_>>::new(audit_writer))
        .wrap(from_fn(begin_audit_chain::<AuditedRequest>))
        .service(target)
}
