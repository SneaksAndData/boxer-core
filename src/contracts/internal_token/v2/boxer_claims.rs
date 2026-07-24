#[cfg(test)]
mod tests;

use crate::contracts::dynamic_claims_collection::DynamicClaims;
use crate::contracts::internal_token::v2::{
    AUDIT_EVENT, PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, VALIDATOR_SCHEMA_ID_KEY,
};
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::validation_service::required_claims::RequiredClaims;
use cedar_policy::{Entity, SchemaFragment};

#[derive(Debug, Clone)]
/// [`BoxerClaims`] represents the claims extracted from an internal token that are necessary for
/// validation and authorization checks.
pub struct BoxerClaims {
    pub schema: SchemaFragment,
    pub schema_id: String,
    pub validator_schema_id: String,
    pub principal: Entity,
    pub audit_event: Option<TokenAuditEvent>,
}

pub trait ToBoxerClaims<T>
where
    T: DynamicClaims,
{
    type Error;
    fn to_boxer_claims(&self) -> Result<BoxerClaims, Self::Error>;
}

impl<T> ToBoxerClaims<T> for T
where
    T: DynamicClaims,
{
    type Error = anyhow::Error;

    fn to_boxer_claims(&self) -> Result<BoxerClaims, Self::Error> {
        let schema = self.get_value(SCHEMA_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;
        let principal = self.get_value(PRINCIPAL_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;
        let schema_id = self
            .get_claim(SCHEMA_ID_KEY)
            .ok_or(anyhow::anyhow!("Missing schema_id"))?;
        let validator_schema_id = self
            .get_claim(VALIDATOR_SCHEMA_ID_KEY)
            .ok_or(anyhow::anyhow!("Missing schema_id"))?;
        let audit_event = self
            .get_value(AUDIT_EVENT)
            .ok_or(anyhow::anyhow!("Missing audit event"))?;

        let schema =
            SchemaFragment::from_json_value(schema.clone()).map_err(|e| anyhow::anyhow!("Invalid schema: {}", e))?;
        let principal = Entity::from_json_value(principal.clone(), None)
            .map_err(|e| anyhow::anyhow!("Invalid principal: {}", e))?;
        let audit_event = serde_json::from_value(audit_event).ok();
        Ok(BoxerClaims {
            schema,
            principal,
            audit_event,
            schema_id,
            validator_schema_id,
        })
    }
}

impl RequiredClaims for BoxerClaims {
    fn get_validator_schema_id(&self) -> String {
        self.validator_schema_id.clone()
    }

    fn get_principal(&self) -> Entity {
        self.principal.clone()
    }

    fn get_schema(&self) -> SchemaFragment {
        self.schema.clone()
    }
}
