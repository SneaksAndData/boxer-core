use crate::contracts::internal_token::v1::token::{InternalToken, TokenMetadata};
use anyhow::anyhow;
use cedar_policy::{Entity, SchemaFragment};
use std::time::Duration;

pub struct InternalTokenBuilder {
    principal: Option<Entity>,
    schema: Option<SchemaFragment>,
    user_id: Option<String>,
    identity_provider: Option<String>,
    schema_name: Option<String>,
    version: String,
    validity_period: Option<Duration>,
    validator_schema_id: Option<String>,
}

impl InternalTokenBuilder {
    pub fn new() -> Self {
        Self {
            principal: None,
            schema: None,
            user_id: None,
            identity_provider: None,
            schema_name: None,
            version: "v1".to_string(),
            validity_period: None, // Default validity period of 1 hour
            validator_schema_id: None,
        }
    }

    pub fn principal(mut self, principal: Entity) -> Self {
        self.principal = Some(principal);
        self
    }

    pub fn schema(mut self, schema: SchemaFragment) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn identity_provider(mut self, identity_provider: String) -> Self {
        self.identity_provider = Some(identity_provider);
        self
    }

    pub fn schema_name(mut self, schema_name: String) -> Self {
        self.schema_name = Some(schema_name);
        self
    }

    pub fn validator_schema_id(mut self, validator_schema_id: String) -> Self {
        self.validator_schema_id = Some(validator_schema_id);
        self
    }

    pub fn validity_period(mut self, validity_period: Duration) -> Self {
        self.validity_period = Some(validity_period);
        self
    }

    pub fn build(self) -> Result<InternalToken, anyhow::Error> {
        let principal = self.principal.ok_or(anyhow!("Principal is required"))?;
        let schema = self.schema.ok_or(anyhow!("Schema is required"))?;
        let user_id = self.user_id.ok_or(anyhow!("User ID is required"))?;
        let identity_provider = self.identity_provider.ok_or(anyhow!("Identity provider is required"))?;
        let schema_name = self.schema_name.ok_or(anyhow!("Schema name is required"))?;
        let validity_period = self.validity_period.ok_or(anyhow!("Validity period is required"))?;
        let validator_schema_id = self
            .validator_schema_id
            .ok_or(anyhow!("Validator schema ID is required"))?;

        Ok(InternalToken {
            principal,
            schema,
            metadata: TokenMetadata {
                user_id,
                identity_provider,
            },
            version: self.version,
            schema_id: schema_name,
            validity_period,
            validator_schema_id,
        })
    }
}

impl InternalToken {
    // Add this method to the existing implementation
    pub fn builder() -> InternalTokenBuilder {
        InternalTokenBuilder::new()
    }
}
