pub mod audit_event_source;
pub mod audit_recorder_factory;
pub mod audit_writer;
#[cfg(test)]
mod tests;

use super::audited_error::AuditedError;
use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::services::audit::AuditService;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;

/// [`AuditRecorder`] is an Actix Web middleware that intercepts incoming requests and outgoing
/// responses to record audit information using the provided [`AuditService`].
pub struct AuditRecorder<NextService, Req> {
    audit_service: Arc<dyn AuditWriter>,
    next: Arc<NextService>,
    phantom: std::marker::PhantomData<Req>,
}

/// The constructor for the middleware
impl<NextService, Req: AuditEventSource> AuditRecorder<NextService, Req> {
    pub fn new(next: Arc<NextService>, audit_service: Arc<dyn AuditWriter>) -> Self {
        AuditRecorder {
            next,
            audit_service,
            phantom: std::marker::PhantomData,
        }
    }
}

/// The implementation of the middleware logic, which processes the request and response to record
/// audit information.
impl<Next, BodyType, AES> Service<ServiceRequest> for AuditRecorder<Next, AES>
where
    Next: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = actix_web::Error> + 'static,
    Next::Future: 'static,
    BodyType: 'static,
    AES: for<'a> TryFrom<&'a ServiceResponse<BodyType>, Error = actix_web::Error> + AuditEventSource,
{
    type Response = ServiceResponse<BodyType>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    forward_ready!(next);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let next = Arc::clone(&self.next);
        let audit_writer = Arc::clone(&self.audit_service);

        let future = async move {
            let result = next.call(req.into()).await;

            match result {
                Ok(response) => {
                    let audited: AES = AES::try_from(&response)?;
                    audit_writer.write(audited.audit_event());
                    Ok(response)
                }

                Err(error) => {
                    match error.as_error::<AuditedError>() {
                        Some(audited_error) => audit_writer.write(audited_error.event.clone()),
                        None => panic!(
                            "Error without audit should not reach audit recorder middleware: {:?}",
                            error
                        ),
                    };
                    Err(error)
                }
            }
        };
        Box::pin(future)
    }
}
