pub mod decryptor;
pub mod request_with_token;
pub mod token_decryptor_middleware_factory;

use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::http::middleware::token_decryptor_middleware::request_with_token::RequestWithToken;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, forward_ready};
use actix_web::error::ErrorBadRequest;
use futures_util::future::LocalBoxFuture;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// Actix middleware that decrypts the token from the request and attaches claims.
pub struct TokenDecryptorMiddleware<Next, D, R> {
    decryptor: Arc<D>,
    next: Rc<Next>,
    phantom_data: PhantomData<R>,
}

impl<Next, Body, D, R> Service<ServiceRequest> for TokenDecryptorMiddleware<Next, D, R>
where
    Next: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = actix_web::Error> + 'static,
    Next::Future: 'static,
    Body: 'static,
    D: Decryptor + 'static,
    R: RequestWithToken + 'static,
{
    type Response = ServiceResponse<Body>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(next);

    /// Decrypts the token in the request, injects the claims, and forwards it downstream.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let decryptor = self.decryptor.clone();
        let next = self.next.clone();
        let future = async move {
            let req = R::try_from_request(req).map_err(ErrorBadRequest)?;
            let encrypted_token = req.token();
            let claims = decryptor
                .decrypt(encrypted_token)
                .map_err(ErrorBadRequest)
                .map_err(|e| AuditedError::new(req.audit_event(), e))?;
            next.call(req.set_claims(claims).map_err(ErrorBadRequest)?).await
        };
        Box::pin(future)
    }
}
