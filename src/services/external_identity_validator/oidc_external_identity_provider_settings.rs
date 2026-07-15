use crate::services::external_identity_validator::ExternalIdentityValidator;
use crate::services::external_identity_validator::oidc_validator::{GenericClaims, OidcValidator};
use crate::services::external_identity_validator_factory::ExternalIdentityValidatorFactory;
use async_trait::async_trait;
use jwt_authorizer::error::InitError;
use jwt_authorizer::{AuthorizerBuilder, JwtAuthorizer, Validation};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OidcExternalIdentityProviderSettings {
    /// The claim that contains the user id (or name) in the external token.
    /// This is used to extract the user id from the token and issue the internal token with
    /// policy based on external identity.
    pub user_id_claim: String,

    /// The well known uri of the identity provider.
    /// This is used to get the public key to validate the token.
    pub discovery_url: String,

    /// The list of issuers that are allowed to issue tokens.
    pub issuers: Vec<String>,

    /// The list of audiences that are allowed to consume tokens.
    pub audiences: Vec<String>,
}

#[async_trait]
impl ExternalIdentityValidatorFactory for OidcExternalIdentityProviderSettings {
    type Error = InitError;

    async fn build_validator(
        self,
        name: String,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, Self::Error> {
        let validation_builder = Validation::new().iss(&self.issuers).aud(&self.audiences);
        let builder: AuthorizerBuilder<GenericClaims> =
            JwtAuthorizer::from_oidc(self.discovery_url.as_str()).validation(validation_builder);
        let authorizer = builder.build().await?;
        Ok(Arc::new(OidcValidator::new(authorizer, self.user_id_claim, name)))
    }
}
