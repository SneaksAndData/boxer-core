use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use crate::services::backends::kubernetes::kubernetes_repository::to_resource::ToResource;
use crate::services::backends::kubernetes::kubernetes_repository::try_from_resource::TryFromResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::legacy::repositories::resource_repository::resource_discovery_document::{
    ResourceDiscoveryDocument, ResourceDiscoveryDocumentSpec,
};
use crate::services::legacy::repositories::resource_repository::resource_set_registration::{
    ResourceRouteRegistration, ResourceSetRegistration,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct SchemaBoundResourceSetRegistration {
    pub hostname: String,
    pub routes: Vec<ResourceRouteRegistration>,
    pub schema: String,
}

impl TryFromResource<ResourceDiscoveryDocument> for SchemaBoundResourceSetRegistration {
    type Error = Status;

    fn try_from_resource(resource: Arc<ResourceDiscoveryDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl ToResource<ResourceDiscoveryDocument> for SchemaBoundResourceSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<ResourceDiscoveryDocument, Status> {
        let spec = ResourceDiscoveryDocumentSpec::try_from(self).unwrap();
        Ok(ResourceDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl Into<ResourceSetRegistration> for SchemaBoundResourceSetRegistration {
    fn into(self) -> ResourceSetRegistration {
        let routes: Vec<ResourceRouteRegistration> = self
            .routes
            .into_iter()
            .map(|route| ResourceRouteRegistration {
                route_template: route.route_template,
                resource_uid: route.resource_uid,
            })
            .collect();

        ResourceSetRegistration {
            hostname: self.hostname,
            routes,
        }
    }
}

impl ToAuditRecord for SchemaBoundResourceSetRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
