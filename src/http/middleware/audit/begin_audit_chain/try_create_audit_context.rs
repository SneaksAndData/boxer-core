use actix_web::dev::ServiceRequest;

/// This trait defines the contract for initializing the audit chain. It abstracts the logic of
/// creating the initial audit context from the incoming request. The implementations of this trait
/// should return an error if the audit chain cannot be initialized or if the request already
/// contains an audit context, preventing the creation of multiple audit contexts for the same request.
pub trait TryCreateAuditContext: Into<ServiceRequest> {
    /// Attempts to create a new audit context from the incoming request. If the request already contains
    fn try_create_audit_context(request: ServiceRequest) -> Result<Self, actix_web::Error>
    where
        Self: Sized;
}
