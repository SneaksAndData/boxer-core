use actix_web::dev::HttpServiceFactory;
use actix_web::web;

pub mod audit_recorder;
pub mod audited_error;
pub mod audited_request;
pub mod begin_audit_chain;
pub mod external_token;

pub fn audit_middleware<S: HttpServiceFactory + 'static>(target: S) -> impl HttpServiceFactory {
    web::scope("")
        .wrap_fn(external_token::<AuditedRequest, actix_web::Error>)
        .service(target)
}
