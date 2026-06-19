use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audit_recorder::AuditRecorder;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;

/// Middleware for audit logging factory
pub struct AuditRecorderFactory<AES> {
    pub audit_service: Arc<dyn AuditWriter>,
    phantom: std::marker::PhantomData<AES>,
}

impl<AES> AuditRecorderFactory<AES> {
    pub fn new(audit_service: Arc<dyn AuditWriter>) -> Self {
        AuditRecorderFactory {
            audit_service,
            phantom: std::marker::PhantomData,
        }
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
    AES: TryFrom<ServiceResponse<BodyType>, Error = actix_web::Error>
        + AuditEventSource
        + Into<ServiceResponse<BodyType>>,
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
