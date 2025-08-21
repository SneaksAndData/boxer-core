use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use anyhow::anyhow;
use futures_util::future::{ready, Ready};

impl FromRequest for BoxerClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = match req.extensions().get::<DynamicClaimsCollection>() {
            None => Err(anyhow!("Missing claims, probably the jwt filter is not in place")),
            Some(c) => BoxerClaims::try_from(c),
        };
        let res = claims.map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()));
        ready(res)
    }
}
