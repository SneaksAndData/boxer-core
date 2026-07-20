use crate::models::external_token::ExternalToken;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::token_service::internal_token_service::external_identity_validator_provider::external_identity_provider::ExternalIdentityProvider;
use anyhow::Result;
use async_trait::async_trait;

pub mod internal_token_service;

#[async_trait]
pub trait TokenService: Send + Sync {
    async fn issue_internal_token(
        &self,
        ip: ExternalIdentityProvider,
        token: ExternalToken,
        chained_audit_event: ChainedAuditEvent,
    ) -> Result<String>;
}
