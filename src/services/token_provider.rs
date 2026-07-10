pub mod encrypted_token_service;
pub mod external_identity;
pub mod principal;
pub mod principal_service;

use crate::services::token_provider::external_identity::ExternalIdentity;
use async_trait::async_trait;

#[async_trait]
/// The TokenProvider crate abstracts the service that used to issue an internal token based on the
/// provided external and identity information.
pub trait TokenProvider: Send + Sync + 'static {
    /// Issues an internal token.
    async fn issue_token(&self, external_token: ExternalIdentity) -> Result<String, anyhow::Error>;
}
