use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use crate::services::backends::kubernetes::kubernetes_repository::to_resource::ToResource;
use crate::services::backends::kubernetes::kubernetes_repository::try_from_resource::TryFromResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::legacy::repositories::action_repository::action_discovery_document::{
    ActionDiscoveryDocument, ActionDiscoveryDocumentSpec,
};
use crate::services::legacy::repositories::action_repository::action_route_registration::{
    ActionRouteRegistration, ActionSetRegistration,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct SchemaBoundActionSetRegistration {
    pub hostname: String,
    pub routes: Vec<ActionRouteRegistration>,
    pub schema: String,
}

impl TryFromResource<ActionDiscoveryDocument> for SchemaBoundActionSetRegistration {
    type Error = Status;

    fn try_from_resource(resource: Arc<ActionDiscoveryDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl ToResource<ActionDiscoveryDocument> for SchemaBoundActionSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<ActionDiscoveryDocument, Status> {
        let spec =
            ActionDiscoveryDocumentSpec::try_from(self).map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(ActionDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl Into<ActionSetRegistration> for SchemaBoundActionSetRegistration {
    fn into(self) -> ActionSetRegistration {
        ActionSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
        }
    }
}

impl ToAuditRecord for SchemaBoundActionSetRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
