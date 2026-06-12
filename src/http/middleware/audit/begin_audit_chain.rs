#[cfg(test)]
mod tests;
pub mod try_create_audit_context;

use actix_web::Error;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use try_create_audit_context::TryCreateAuditContext;

/// Middleware to initialize the audit chain for incoming requests.
/// This should be the first middleware in the audit chain to ensure that all subsequent middleware
/// and handlers have access to the audit context.
pub async fn begin_audit_chain<AuditContext: TryCreateAuditContext>(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let audited_request = AuditContext::try_create_audit_context(req)?;
    next.call(audited_request.into()).await
}
