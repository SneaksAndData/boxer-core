#[cfg(test)]
mod tests;

use crate::http;
use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::http::header::HeaderValue;

/// Represents an external JWT Token used to authorize the `ExternalIdentity` and issue an `InternalToken`
#[derive(Clone)]
pub struct ExternalToken(String);

/// Allows `ExternalToken` to be converted to a String
impl Into<String> for ExternalToken {
    fn into(self) -> String {
        self.0
    }
}

/// Allows a String to be converted to an `ExternalToken`
impl From<String> for ExternalToken {
    fn from(token: String) -> Self {
        ExternalToken(token)
    }
}

/// Allows a `HeaderValue` to be converted to an `ExternalToken` if it follows the expected
/// "Bearer <token>" format
impl TryFrom<HeaderValue> for ExternalToken {
    type Error = anyhow::Error;

    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        let string_value = http::extract_bearer_token(&value)?;
        Ok(ExternalToken(string_value))
    }
}

impl TokenWithId for ExternalToken {
    fn id(&self) -> String {
        let token_hash = md5::compute(self.0.as_bytes());
        format!("md5:{:x}", token_hash)
    }
}
