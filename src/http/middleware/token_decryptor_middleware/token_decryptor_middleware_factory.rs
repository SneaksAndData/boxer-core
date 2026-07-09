use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::http::middleware::token_decryptor_middleware::TokenDecryptorMiddleware;
use crate::http::middleware::token_decryptor_middleware::request_with_token::RequestWithToken;
use crate::http::middleware::token_decryptor_middleware::TokenDecryptorMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// Factory that builds the [`TokenDecryptorMiddleware`]
pub struct TokenDecryptorMiddlewareFactory<D, R> {
    decryptor: Arc<D>,
    phantom_data: PhantomData<R>,
}

impl<D, R> TokenDecryptorMiddlewareFactory<D, R> {
    /// Creates a new decrypt-token middleware factory from the provided decryptor.
    pub fn new(decryptor: Arc<D>) -> Self {
        Self {
            decryptor,
            phantom_data: PhantomData,
        }
    }
}

impl<Next, Body, D, R> Transform<Next, ServiceRequest> for TokenDecryptorMiddlewareFactory<D, R>
where
    Next: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = actix_web::Error> + 'static,
    Next::Future: 'static,
    Body: 'static,
    R: RequestWithToken + 'static,
    D: Decryptor + 'static,
{
    type Response = ServiceResponse<Body>;
    type Error = actix_web::Error;
    type Transform = TokenDecryptorMiddleware<Next, D, R>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Wraps the next service in a decrypt-token middleware instance.
    fn new_transform(&self, next: Next) -> Self::Future {
        ready(Ok(TokenDecryptorMiddleware {
            decryptor: self.decryptor.clone(),
            next: Rc::new(next),
            phantom_data: PhantomData,
        }))
    }
}
