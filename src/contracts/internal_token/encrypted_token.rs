use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::http::header::HeaderValue;
use anyhow::anyhow;
use log::debug;

/// Wraps the raw encrypted token value extracted from request headers.
#[derive(Clone)]
pub struct EncryptedToken(String);

impl EncryptedToken {
    fn from_bearer(value: &str) -> Result<Self, anyhow::Error> {
        let parts = value.split(' ').collect::<Vec<&str>>();

        if parts.len() < 2 || parts[0] != "Bearer" {
            debug!("Invalid token received: '{}'", value);
            return Err(anyhow!("Invalid token format"));
        }

        let token = parts.last().copied().unwrap_or_default();
        if token.is_empty() {
            debug!("Invalid token received: '{}'", value);
            return Err(anyhow!("Invalid token format"));
        }

        Ok(EncryptedToken(token.to_string()))
    }
}

impl TryFrom<HeaderValue> for EncryptedToken {
    type Error = anyhow::Error;

    /// Builds an [`EncryptedToken`] from an HTTP header value.
    ///
    /// Returns an error when the header value contains invalid UTF-8.
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        let string_value = value.to_str().map_err(|_| anyhow!("Invalid token format"))?;
        EncryptedToken::from_bearer(string_value)
    }
}

impl TokenWithId for EncryptedToken {
    /// Returns a stable MD5-based identifier for the encrypted token.
    fn id(&self) -> String {
        let token_hash = md5::compute(&self.0);
        format!("md5:{:x}", token_hash)
    }
}

impl Into<String> for EncryptedToken {
    /// Returns the underlying raw encrypted token from the [`EncryptedToken`] instance.
    fn into(self) -> String {
        self.0
    }
}
