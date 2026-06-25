#[cfg(test)]
mod tests;

use crate::http::middleware::extract_external_token::token_with_id::TokenWithId;
use actix_web::http::header::HeaderValue;
use anyhow::bail;

/// Represents an external JWT Token used to authorize the `ExternalIdentity` and issue an `InternalToken`
#[derive(Clone)]
pub struct ExternalToken(String);

/// Allows `ExternalToken` to be converted to a String
impl Into<String> for ExternalToken {
    fn into(self) -> String {
        self.0.clone()
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

    #[allow(unreachable_code)] // False detect
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        match value.to_str() {
            Ok(string_value) => {
                let tokens = string_value.split(" ").collect::<Vec<&str>>();

                if tokens.len() != 2 {
                    return Err(bail!("Invalid token format"));
                }

                if tokens[0] != "Bearer" {
                    return Err(bail!("Invalid token format"));
                }

                Ok(ExternalToken::from(tokens[1].to_owned()))
            }
            Err(_) => bail!("Invalid token format"),
        }
    }
}

impl TokenWithId for ExternalToken {
    fn id(&self) -> String {
        let token_hash = md5::compute(&self.0);
        format!("md5:{:x}", token_hash)
    }
}
