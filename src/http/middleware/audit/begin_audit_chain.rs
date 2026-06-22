#[cfg(test)]
mod tests;
pub mod try_create_audit_context;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage};
use try_create_audit_context::TryCreateAuditContext;

/// Initializes audit context for an incoming request and forwards it down the chain.
///
/// Place this as the first middleware that must run on request ingress so later
/// middleware and handlers can read the audit context from request extensions.
pub async fn begin_audit_chain<AuditContext: TryCreateAuditContext + 'static>(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let audited_request = AuditContext::try_create_audit_context(req)?;
    next.call(audited_request.into()).await
}
