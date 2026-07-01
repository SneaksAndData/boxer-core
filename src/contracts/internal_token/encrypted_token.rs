use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::http::header::HeaderValue;

/// Wraps the raw encrypted token value extracted from request headers.
pub struct EncryptedToken(String);

impl TryFrom<HeaderValue> for EncryptedToken {
    type Error = anyhow::Error;

    /// Builds an [`EncryptedToken`] from an HTTP header value.
    ///
    /// Returns an error when the header value contains invalid UTF-8.
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        Ok(EncryptedToken(value.to_str()?.to_string()))
    }
}

impl TokenWithId for EncryptedToken {
    /// Returns a stable MD5-based identifier for the encrypted token.
    fn id(&self) -> String {
        let token_hash = md5::compute(&self.0);
        format!("md5:{:x}", token_hash)
    }
}
