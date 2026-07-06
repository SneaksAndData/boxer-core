#[cfg(test)]
mod tests;

use crate::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use actix_web::error::ErrorBadRequest;
use actix_web::http::Uri;
use actix_web::{FromRequest, HttpRequest};
use cedar_policy::{EntityId, EntityTypeName, EntityUid};
use std::future::{ready, Ready};
use std::str::FromStr;

const ORIGINAL_URL_NGINX_HEADER: &str = "X-Original-URL";
const ORIGINAL_METHOD_NGINX_HEADER: &str = "X-Original-Method";

const ORIGINAL_METHOD_TRAEFIK_HEADER: &str = "X-Forwarded-Method";
const ORIGINAL_PROTOCOL_TRAEFIK_HEADER: &str = "X-Forwarded-Proto";
const ORIGINAL_HOST_TRAEFIK_HEADER: &str = "X-Forwarded-Host";
const ORIGINAL_URL_TRAEFIK_HEADER: &str = "X-Forwarded-Uri";

/// Contains the request context for validation.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub original_url: String,
    pub original_method: String,
}

impl RequestContext {
    /// Creates a new `RequestContext` with the given original URL and method.
    pub fn new(original_url: String, original_method: String) -> RequestContext {
        RequestContext {
            original_url,
            original_method,
        }
    }

    /// Converts this request into a Cedar `EntityUid` representing the HTTP resource.
    pub fn to_resource(&self) -> anyhow::Result<EntityUid> {
        let tp = EntityTypeName::from_str("Http")?;
        let uri = self.original_url.parse::<Uri>()?;
        let name = uri.host().unwrap().to_string() + uri.path();
        let n = EntityId::from_str(&name)?;
        Ok(EntityUid::from_type_name_and_id(tp, n))
    }
}

impl FromRequest for RequestContext {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let cx = start_trace("extract_request_context", None);
        let result = extract_headers(req)
            .stop_trace(cx)
            .map(|(url, method)| RequestContext::new(url, method))
            .map_err(|err| ErrorBadRequest(err.to_string()));

        ready(result)
    }
}

fn extract_headers(req: &HttpRequest) -> anyhow::Result<(String, String)> {
    let original_url = extract_url(req, ORIGINAL_URL_NGINX_HEADER)?;
    let original_method = extract_header(req, ORIGINAL_METHOD_NGINX_HEADER, ORIGINAL_METHOD_TRAEFIK_HEADER)?;
    Ok((original_url, original_method))
}

fn extract_header(req: &HttpRequest, header_name: &'static str, fallback: &'static str) -> anyhow::Result<String> {
    let header = req.headers().get(header_name);
    let fallback_header = req.headers().get(fallback);
    let result = match header {
        Some(value) => Some(value.to_str()?.to_string()),
        None => match fallback_header {
            Some(value) => Some(value.to_str()?.to_string()),
            None => None,
        },
    };

    Ok(result.ok_or_else(|| anyhow::anyhow!("Missing required header: {header_name} or {fallback} in request"))?)
}

fn extract_url(req: &HttpRequest, header_name: &'static str) -> anyhow::Result<String> {
    let header = req.headers().get(header_name);
    let result = match header {
        Some(value) => Some(value.to_str()?.to_string()),
        None => match extract_traefik_headers(req) {
            Some(value) => Some(value),
            None => None,
        },
    };

    Ok(result.ok_or_else(|| anyhow::anyhow!("Missing required headers"))?)
}

fn extract_traefik_headers(req: &HttpRequest) -> Option<String> {
    let headers = req.headers();

    let proto = headers.get(ORIGINAL_PROTOCOL_TRAEFIK_HEADER)?.to_str().ok()?;
    let host = headers.get(ORIGINAL_HOST_TRAEFIK_HEADER)?.to_str().ok()?;
    let uri = headers.get(ORIGINAL_URL_TRAEFIK_HEADER)?.to_str().ok()?;

    let uri = uri.trim_start_matches('/');
    Some(format!("{proto}://{host}/{uri}"))
}
