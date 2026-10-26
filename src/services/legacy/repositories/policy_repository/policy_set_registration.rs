use crate::services::legacy::repositories::policy_repository::schema_bound_policy_set_registration::SchemaBoundPolicySetRegistration;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PolicySetRegistration {
    pub policy: String,
}

impl PolicySetRegistration {
    pub fn with_schema(self, schema: String) -> SchemaBoundPolicySetRegistration {
        SchemaBoundPolicySetRegistration {
            policy: self.policy.clone(),
            schema,
        }
    }
}
