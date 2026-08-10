use actix_web::http::header::HeaderValue;
use anyhow::anyhow;
use log::debug;

pub mod conversions;
pub mod middleware;
pub mod readiness;

/// Extracts the Bearer token from the header value.
pub(crate) fn extract_bearer_token(value: &HeaderValue) -> Result<String, anyhow::Error> {
    let string_value = value
        .to_str()
        .map_err(|e| anyhow!("Failed to extract data from : {}", e))?;
    let parts = string_value.split(' ').collect::<Vec<&str>>();

    if parts.len() < 2 || parts[0] != "Bearer" {
        debug!("Invalid token received: '{}'", string_value);
        return Err(anyhow!("Invalid header format. Expected `Bearer ...`"));
    }

    let token = parts.last().copied().unwrap_or_default();
    if token.is_empty() {
        debug!("Invalid token received: '{}'", string_value);
        return Err(anyhow!("Invalid header format. Expected `Bearer ...`"));
    }

    Ok(token.to_string())
}
