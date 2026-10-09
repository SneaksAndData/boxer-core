use crate::http::middleware::audit::audit_recorder::audit_recorder_factory::AuditRecorderFactory;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audit_scope::errors_as_unauthorized::errors_as_unauthorized;
use crate::http::middleware::audit::audit_scope::ok_or_unauthorized_statuses::OkOrUnauthorized;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::audited_response::AuditedResponse;
use crate::http::middleware::audit::begin_audit_chain::begin_audit_chain;
use crate::http::middleware::audit::enrich_from_origin::enrich_from_origin;
use crate::http::middleware::audit::external_request::ExternalRequest;
use crate::http::middleware::audit::finalize_on_status;
use crate::http::middleware::audit::internal_request::InternalRequest;
use crate::http::middleware::audit::status_filter::finalize_on_fail::finalize_on_fail;
use crate::http::middleware::audit::status_filter::skip_unmatched_statuses::SkipUnmatched;
use crate::http::middleware::extract_external_token::extract_external_token;
use crate::http::middleware::extract_internal_token::extract_encrypted_token;
use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::http::middleware::token_decryptor_middleware::token_decryptor_middleware_factory::TokenDecryptorMiddlewareFactory;
use actix_web::Scope;
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use std::sync::Arc;

mod errors_as_unauthorized;
mod ok_or_unauthorized_statuses;

/// Extension trait for attaching the complete audit middleware chain to an Actix [`Scope`].
///
/// The resulting scope initializes an audit event, extracts the external token,
/// and records a final audit event through the provided [`AuditWriter`].
pub trait AuditScope {
    /// Wraps this scope with the audit middleware pipeline.
    ///
    ///
    /// Middleware order is significant:
    /// - starts the audit chain (`begin_audit_chain`),
    /// - extracts and validates the external token (`extract_external_token`),
    /// - enriches final events on the response path (`enrich_from_origin`),
    /// - records the terminal audit event (`AuditRecorderFactory`).
    fn with_initial_audit_scope(self, writer: Arc<dyn AuditWriter>) -> impl HttpServiceFactory;

    /// Wraps this scope with the continuation audit middleware pipeline.
    ///
    /// Use this on internal routes where the external token is already present and
    /// an internal token must be extracted/decrypted before recording the final audit event.
    /// Handler responses other than 200 and 401 finalize any unfinished audit event.
    /// Propagated service errors are wrapped with HTTP status 401 after audit recording;
    /// their response bodies and body framing/encoding headers are removed. Successful service results are unchanged.
    ///
    /// Middleware order is significant:
    /// - extracts the encrypted internal token (`extract_encrypted_token`),
    /// - decrypts and enriches request context (`TokenDecryptorMiddlewareFactory`),
    /// - enriches final events on the response path (`enrich_from_origin`),
    /// - records the terminal audit event (`AuditRecorderFactory`).
    fn continue_audit_scope<D>(self, writer: Arc<dyn AuditWriter>, decryptor: Arc<D>) -> impl HttpServiceFactory
    where
        D: Decryptor + 'static;
}

impl AuditScope for Scope {
    fn with_initial_audit_scope(self, writer: Arc<dyn AuditWriter>) -> impl HttpServiceFactory {
        self.wrap(from_fn(extract_external_token::<ExternalRequest, AuditedError>))
            .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>))
            .wrap(from_fn(finalize_on_fail))
            .wrap(from_fn(begin_audit_chain::<ExternalRequest>))
            .wrap(AuditRecorderFactory::<AuditedResponse<_>>::new(writer))
            .wrap(from_fn(errors_as_unauthorized))
    }

    fn continue_audit_scope<D>(self, writer: Arc<dyn AuditWriter>, decryptor: Arc<D>) -> impl HttpServiceFactory
    where
        D: Decryptor + 'static,
    {
        self.wrap(TokenDecryptorMiddlewareFactory::<D, InternalRequest>::new(decryptor))
            .wrap(from_fn(extract_encrypted_token::<InternalRequest, AuditedError>))
            .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>))
            .wrap(AuditRecorderFactory::<AuditedResponse<_>>::new(writer))
            .wrap(from_fn(begin_audit_chain::<InternalRequest>))
            .wrap(from_fn(finalize_on_status::<SkipUnmatched<OkOrUnauthorized>>))
            .wrap(from_fn(errors_as_unauthorized))
    }
}
