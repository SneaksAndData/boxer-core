use crate::models::external_token::ExternalToken;
use async_trait::async_trait;
use external_identity::ExternalIdentity;

pub mod external_identity;
pub mod oidc_external_identity_provider_settings;
pub mod oidc_validator;

/// Validator for external identity.
#[async_trait]
pub trait ExternalIdentityValidator {
    /// Validate the external identity token and return the external identity.
    async fn validate(&self, token: ExternalToken) -> Result<ExternalIdentity, anyhow::Error>;
}
