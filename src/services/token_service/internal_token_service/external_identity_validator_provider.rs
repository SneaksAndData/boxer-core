pub mod external_identity_provider;

use crate::services::external_identity_validator::ExternalIdentityValidator;
use crate::services::token_service::internal_token_service::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use async_trait::async_trait;
use std::sync::Arc;

/// Read-only interface for managing external identity validators.
#[async_trait]
pub trait ExternalIdentityValidatorProvider: Send + Sync {
    async fn get(
        &self,
        provider: ExternalIdentityProvider,
    ) -> Result<Arc<dyn ExternalIdentityValidator + Send + Sync>, anyhow::Error>;
}
