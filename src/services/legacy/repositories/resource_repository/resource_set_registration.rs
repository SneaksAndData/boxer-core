use crate::services::legacy::repositories::resource_repository::schema_bound_resource_set_registration::SchemaBoundResourceSetRegistration;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ResourceRouteRegistration {
    pub route_template: String,
    pub resource_uid: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ResourceSetRegistration {
    pub hostname: String,
    pub routes: Vec<ResourceRouteRegistration>,
}

impl ResourceSetRegistration {
    pub fn with_schema(self, schema: String) -> SchemaBoundResourceSetRegistration {
        SchemaBoundResourceSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
            schema,
        }
    }
}
