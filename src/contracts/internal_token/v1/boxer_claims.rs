use crate::contracts::dynamic_claims_collection::{get_claim, get_value, DynamicClaimsCollection};
use crate::contracts::internal_token::v1::{PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, VALIDATOR_SCHEMA_ID_KEY};
use cedar_policy::{Entity, SchemaFragment};

#[derive(Debug)]
pub struct BoxerClaims {
    pub schema: SchemaFragment,
    pub schema_id: String,
    pub validator_schema_id: String,
    pub principal: Entity,
}

impl TryFrom<&DynamicClaimsCollection> for BoxerClaims {
    type Error = anyhow::Error;

    fn try_from(c: &DynamicClaimsCollection) -> Result<Self, Self::Error> {
        let schema = get_value(c, SCHEMA_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;
        let principal = get_value(c, PRINCIPAL_KEY).ok_or(anyhow::anyhow!("Missing schema"))?;
        let schema_id = get_claim(c, SCHEMA_ID_KEY).ok_or(anyhow::anyhow!("Missing schema_id"))?;
        let validator_schema_id = get_claim(c, VALIDATOR_SCHEMA_ID_KEY).ok_or(anyhow::anyhow!("Missing schema_id"))?;

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
