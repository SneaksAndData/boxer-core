use crate::models::external_token::ExternalToken;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::token_service::internal_token_service::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use crate::services::token_service::TokenService;
use anyhow::Result;
use async_trait::async_trait;
use external_identity_validator_provider::ExternalIdentityValidatorProvider;
use std::sync::Arc;
use token_provider::TokenProvider;

pub mod external_identity_validator_provider;
pub mod token_provider;

pub struct InternalTokenService {
    validator_provider: Arc<dyn ExternalIdentityValidatorProvider>,
    token_provider: Arc<dyn TokenProvider>,
}

impl InternalTokenService {
    /// Creates an instance of `InternalTokenService`.
    pub fn new(
        validator_provider: Arc<dyn ExternalIdentityValidatorProvider>,
        token_provider: Arc<dyn TokenProvider>,
    ) -> Self {
        Self {
            validator_provider,
            token_provider,
        }
    }
}

#[async_trait]
impl TokenService for InternalTokenService {
    async fn issue_internal_token(
        &self,
        ip: ExternalIdentityProvider,
        token: ExternalToken,
        chained_audit_event: ChainedAuditEvent,
    ) -> Result<String> {
        let validator = self.validator_provider.get(ip).await?;
        let external_identity = validator.validate(token).await?;
        let internal_token = self
            .token_provider
            .issue_token(external_identity, chained_audit_event)
            .await?;
        Ok(internal_token)
    }
}
