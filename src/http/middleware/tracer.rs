pub mod tracer_middleware_factory;

use crate::services::observability::open_telemetry::tracing::start_trace;
use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, forward_ready};
use futures_util::future::LocalBoxFuture;
use opentelemetry::context::FutureExt;
use std::sync::Arc;

/// Middleware to initialize the audit chain for incoming requests.
/// This should be the first middleware in the audit chain to ensure that all subsequent middleware
/// and handlers have access to the audit context.
pub struct TraceMiddleware<Next> {
    next: Arc<Next>,
    span_name: &'static str,
}

impl<Next, BodyType> Service<ServiceRequest> for TraceMiddleware<Next>
where
    Next: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    Next::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(next);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let span_name = self.span_name;
        let next = self.next.clone();
        Box::pin(async move {
            let parent = start_trace(span_name, None);
            next.call(request).with_context(parent.clone()).await
        })
    }
}
