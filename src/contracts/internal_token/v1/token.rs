use crate::contracts::internal_token::API_VERSION_KEY;
use crate::contracts::internal_token::v1::{
    BOXER_AUDIENCE, BOXER_ISSUER, IDENTITY_PROVIDER_KEY, PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, USER_ID_KEY,
    VALIDATOR_SCHEMA_ID_KEY,
};
use cedar_policy::{Entity, SchemaFragment};
use jwt::Claims;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod internal_token_builder;

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
    pub user_id: String,
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
                user_id,
                identity_provider: external_identity_provider,
            },
            version: "v1".to_string(),
            schema_id,
            validity_period,
            validator_schema_id,
        }
    }
}

impl TryInto<Claims> for InternalToken {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Claims, Self::Error> {
        let mut claims: Claims = Default::default();
        claims.private.insert(API_VERSION_KEY.to_string(), self.version.into());
        claims
            .private
            .insert(PRINCIPAL_KEY.to_string(), self.principal.to_json_value()?);
        claims
            .private
            .insert(SCHEMA_KEY.to_string(), self.schema.to_json_value()?);
        claims
            .private
            .insert(USER_ID_KEY.to_string(), self.metadata.user_id.into());
        claims.private.insert(
            IDENTITY_PROVIDER_KEY.to_string(),
            self.metadata.identity_provider.into(),
        );
        claims.private.insert(SCHEMA_ID_KEY.to_string(), self.schema_id.into());
        claims
            .private
            .insert(VALIDATOR_SCHEMA_ID_KEY.to_string(), self.validator_schema_id.into());

        claims.registered.issuer = Some(BOXER_ISSUER.to_string());
        claims.registered.audience = Some(BOXER_AUDIENCE.to_string());

        let one_hour = SystemTime::now() + self.validity_period;
        claims.registered.expiration = Some(one_hour.duration_since(UNIX_EPOCH)?.as_secs());
        Ok(claims)
    }
}
