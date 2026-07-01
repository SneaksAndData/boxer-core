use crate::http::middleware::audit::audit_recorder::audit_recorder_factory::AuditRecorderFactory;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::audited_response::AuditedResponse;
use crate::http::middleware::audit::begin_audit_chain::begin_audit_chain;
use crate::http::middleware::audit::external_request::ExternalRequest;
use crate::http::middleware::extract_external_token::extract_external_token;
use actix_web::Scope;
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use std::sync::Arc;

/// Extension trait for attaching the complete audit middleware chain to an Actix [`Scope`].
///
/// The resulting scope initializes an audit event, extracts the external token,
/// and records a final audit event through the provided [`AuditWriter`].
pub trait AuditScope {
    /// Wraps this scope with the audit middleware pipeline.
    ///
    /// Middleware order is significant:
    /// - starts the audit chain (`begin_audit_chain`),
    /// - extracts and validates the external token (`extract_external_token`),
    /// - records the terminal audit event (`AuditRecorderFactory`).
    fn with_initial_audit_scope(self, writer: Arc<dyn AuditWriter>) -> impl HttpServiceFactory;
}

impl AuditScope for Scope {
    fn with_initial_audit_scope(self, writer: Arc<dyn AuditWriter>) -> impl HttpServiceFactory {
        self.wrap(from_fn(extract_external_token::<ExternalRequest, AuditedError>))
            .wrap(AuditRecorderFactory::<AuditedResponse<_>>::new(writer))
            .wrap(from_fn(begin_audit_chain::<ExternalRequest>))
    }
}
