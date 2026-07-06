use actix_web::http::Uri;
use cedar_policy::{EntityId, EntityTypeName, EntityUid};
use std::str::FromStr;

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
