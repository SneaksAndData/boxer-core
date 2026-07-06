pub mod cedar_validation_service;
pub mod http_method;
pub mod path_segment;
pub mod request_context;
pub mod request_segment;
pub mod required_claims;
pub mod schema_provider;

use crate::services::validation_service::request_context::RequestContext;
use crate::services::validation_service::required_claims::RequiredClaims;
use async_trait::async_trait;

/// Validates the Claims and RequestContext against the policies and schemas defined in the system.
#[async_trait]
pub trait ValidationService<Claims: RequiredClaims>: Send + Sync {
    async fn validate(&self, boxer_claims: Claims, request_context: RequestContext) -> Result<(), anyhow::Error>;
}
