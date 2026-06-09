use crate::http::middleware::audit::audit_recorder::AuditRecorder;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::services::audit::AuditService;
use crate::services::audit::log_audit_service::LogAuditService;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;

/// Middleware for audit logging factory
pub struct AuditRecorderFactory<AES> {
    pub audit_service: Arc<dyn AuditService>,
    phantom: std::marker::PhantomData<AES>,
}

impl<AES> AuditRecorderFactory<AES> {
    pub fn new(audit_service: Arc<dyn AuditService>) -> Self {
        AuditRecorderFactory {
            audit_service,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<AES> Default for AuditRecorderFactory<AES> {
    fn default() -> Self {
        Self::new(Arc::new(LogAuditService::new()))
    }
}

/// Transform trait implementation
/// `NextServiceType` - type of the next service
/// `BodyType` - type of response's body
impl<NextService, BodyType, AES> Transform<NextService, ServiceRequest> for AuditRecorderFactory<AES>
where
    NextService: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = actix_web::Error> + 'static,
    NextService::Future: 'static,
    BodyType: 'static,
    AES: for<'a> TryFrom<&'a ServiceResponse<BodyType>, Error = actix_web::Error> + AuditEventSource,
{
    type Response = ServiceResponse<BodyType>;
    type Error = actix_web::Error;
    type Transform = AuditRecorder<NextService, AES>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<AuditRecorder<NextService, AES>, Self::InitError>>;

    fn new_transform(&self, service: NextService) -> Self::Future {
        let audit_service = self.audit_service.clone();
        Box::pin(async move { Ok(AuditRecorder::new(Arc::new(service), audit_service)) })
    }
}
