use super::TraceMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;

/// Factory for the tracer middleware
pub struct TracerMiddlewareFactory {
    span_name: &'static str,
}

impl TracerMiddlewareFactory {
    /// The Tracer middleware constructor
    /// parameter `span_name` - Name of the span for the tracer
    pub fn new(span_name: &'static str) -> Self {
        TracerMiddlewareFactory { span_name }
    }
}

impl<Next, Body> Transform<Next, ServiceRequest> for TracerMiddlewareFactory
where
    Next: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = Error> + 'static,
    Next::Future: 'static,
    Body: 'static,
{
    type Response = ServiceResponse<Body>;
    type Error = Error;
    type Transform = TraceMiddleware<Next>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<TraceMiddleware<Next>, Self::InitError>>;

    fn new_transform(&self, next: Next) -> Self::Future {
        let span_name = self.span_name;
        Box::pin(async move {
            let mw = TraceMiddleware {
                next: Arc::new(next),
                span_name,
            };
            Ok(mw)
        })
    }
}
