#[cfg(test)]
mod tests;

use crate::contracts::internal_token::API_VERSION_KEY;
use crate::contracts::internal_token::v1::{
    BOXER_AUDIENCE, BOXER_ISSUER, IDENTITY_PROVIDER_KEY, PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, USER_ID_KEY,
    VALIDATOR_SCHEMA_ID_KEY,
};
use cedar_policy::{Entity, SchemaFragment};
use josekit::jwt::JwtPayload;
use std::time::{Duration, SystemTime};

pub struct InternalToken {
    pub principal: Entity,
    pub schema: SchemaFragment,
    pub schema_id: String,
    pub metadata: TokenMetadata,
    pub version: String,
    pub validity_period: Duration,
    pub validator_schema_id: String,
}

pub struct TokenMetadata {
    pub external_identity: String,
    pub identity_provider: String,
}

impl InternalToken {
    pub fn new(
        principal: Entity,
        schema: SchemaFragment,
        user_id: String,
        external_identity_provider: String,
        schema_id: String,
        validity_period: Duration,
        validator_schema_id: String,
    ) -> Self {
        InternalToken {
            principal,
            schema,
            metadata: TokenMetadata {
                external_identity: user_id,
                identity_provider: external_identity_provider,
            },
            version: "v1".to_string(),
            schema_id,
            validity_period,
            validator_schema_id,
        }
    }
}

impl TryInto<JwtPayload> for InternalToken {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<JwtPayload, Self::Error> {
        let mut claims: JwtPayload = Default::default();
        claims.set_claim(API_VERSION_KEY, Some(self.version.into()))?;
        claims.set_claim(PRINCIPAL_KEY, Some(self.principal.to_json_value()?))?;
        claims.set_claim(SCHEMA_KEY, Some(self.schema.to_json_value()?))?;
        claims.set_claim(USER_ID_KEY, Some(self.metadata.external_identity.into()))?;
        claims.set_claim(IDENTITY_PROVIDER_KEY, Some(self.metadata.identity_provider.into()))?;
        claims.set_claim(SCHEMA_ID_KEY, Some(self.schema_id.into()))?;
        claims.set_claim(VALIDATOR_SCHEMA_ID_KEY, Some(self.validator_schema_id.into()))?;

        claims.set_issuer(BOXER_ISSUER.to_string());
        claims.set_audience(vec![BOXER_AUDIENCE.to_string()]);

        let one_hour = SystemTime::now() + self.validity_period;
        claims.set_expires_at(&one_hour);
        Ok(claims)
    }
}
