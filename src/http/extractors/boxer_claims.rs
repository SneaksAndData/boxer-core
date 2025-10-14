use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::v1::boxer_claims::{BoxerClaims, ToBoxerClaims};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use anyhow::anyhow;
use futures_util::future::{Ready, ready};

impl FromRequest for BoxerClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = match req.extensions().get::<DynamicClaimsCollection>() {
            None => Err(anyhow!("Missing claims, probably the jwt filter is not in place")),
            Some(c) => c.to_boxer_claims(),
        };
        let res = claims.map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()));
        ready(res)
    }
}
