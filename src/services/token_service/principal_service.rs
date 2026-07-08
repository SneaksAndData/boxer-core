use crate::services::token_service::external_identity::ExternalIdentity;
use crate::services::token_service::principal::Principal;
use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::SchemaFragment;

/// Manages an information about the principal in the system.
/// It provides methods to retrieve principal information, validator schemas, and schema
/// fragments based on external identity.
#[async_trait]
pub trait PrincipalService: Send + Sync + 'static {
    /// Retrieves an internal principal information.
    async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal>;

    /// Returns a validator schema associated to the principal.
    async fn get_validator_schema(&self, external_identity: ExternalIdentity) -> Result<String>;

    /// Return schema fragments that should be used to validate the principal on the
    /// validator side.
    async fn get_schemas(&self, schema_id: String) -> Result<SchemaFragment, anyhow::Error>;
}
