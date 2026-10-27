use crate::services::legacy::repositories::action_repository::schema_bound_action_route_registration::SchemaBoundActionSetRegistration;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ActionRouteRegistration {
    pub method: String,
    pub route_template: String,
    pub action_uid: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ActionSetRegistration {
    pub hostname: String,
    pub routes: Vec<ActionRouteRegistration>,
}
impl ActionSetRegistration {
    pub fn with_schema(self, schema: String) -> SchemaBoundActionSetRegistration {
        SchemaBoundActionSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
            schema,
        }
    }
}
