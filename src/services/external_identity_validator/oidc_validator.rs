use crate::models::external_token::ExternalToken;
use crate::services::external_identity_validator::ExternalIdentityValidator;
use crate::services::external_identity_validator::external_identity::ExternalIdentity;
use anyhow::bail;
use async_trait::async_trait;
use jwt_authorizer::Authorizer;
use log::info;
use serde_json::Value;
use std::collections::HashMap;

pub(super) type GenericClaims = HashMap<String, Value>;

pub(super) struct OidcValidator {
    authorizer: Authorizer<GenericClaims>,
    user_id_claim: String,
    name: String,
}

#[async_trait]
impl ExternalIdentityValidator for OidcValidator {
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error> {
        let token_str: String = token.into();
        let result = self.authorizer.check_auth(&token_str).await?;
        let maybe_ext_id = self.extract_user_id(&result.claims, &self.user_id_claim, self.name.clone());
        match maybe_ext_id {
            Some(ext_id) => {
                info!(
                    "Successfully validated token for user {}/{}",
                    self.name,
                    ext_id.user_id()
                );
                Ok(ext_id)
            }
            None => bail!("Failed to extract user id from token"),
        }
    }
}

impl OidcValidator {
    pub fn new(authorizer: Authorizer<GenericClaims>, user_id_claim: String, name: String) -> Self {
        OidcValidator {
            authorizer,
            user_id_claim,
            name,
        }
    }

    fn extract_user_id(
        &self,
        claims: &GenericClaims,
        user_id_claim: &str,
        identity_provider: String,
    ) -> Option<ExternalIdentity> {
        let value = claims.get(user_id_claim)?;
        let user_id = value.as_str()?.to_owned();
        Some(ExternalIdentity::new(identity_provider, user_id))
    }
}
