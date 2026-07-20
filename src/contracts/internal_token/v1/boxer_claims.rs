#[cfg(test)]
mod tests;

use crate::contracts::dynamic_claims_collection::DynamicClaims;
use crate::contracts::internal_token::v1::{PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, VALIDATOR_SCHEMA_ID_KEY};
use crate::contracts::internal_token::v2::boxer_claims::BoxerClaims as V2BoxerClaims;
use crate::http::middleware::audit::internal_request::upgrade_version::UpgradeVersion;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::validation_service::required_claims::RequiredClaims;
use cedar_policy::{Entity, SchemaFragment};

#[derive(Debug)]
pub struct BoxerClaims {
    pub schema: SchemaFragment,
    pub schema_id: String,
    pub validator_schema_id: String,
    pub principal: Entity,
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

        Ok(BoxerClaims {
            schema: SchemaFragment::from_json_value(schema.clone())
                .map_err(|e| anyhow::anyhow!("Invalid schema: {}", e))?,
            principal: Entity::from_json_value(principal.clone(), None)
                .map_err(|e| anyhow::anyhow!("Invalid principal: {}", e))?,
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

impl UpgradeVersion for BoxerClaims {
    type Result = V2BoxerClaims;

    type Context = Option<TokenAuditEvent>;

    fn upgrade_version(self, audit_event: Self::Context) -> V2BoxerClaims {
        V2BoxerClaims {
            schema: self.schema,
            schema_id: self.schema_id,
            validator_schema_id: self.validator_schema_id,
            principal: self.principal,
            audit_event,
        }
    }
}
