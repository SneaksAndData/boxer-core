use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use crate::services::backends::kubernetes::kubernetes_repository::to_resource::ToResource;
use crate::services::backends::kubernetes::kubernetes_repository::try_from_resource::TryFromResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::legacy::repositories::policy_repository::policy_document::{PolicyDocument, PolicyDocumentSpec};
use crate::services::legacy::repositories::policy_repository::policy_set_registration::PolicySetRegistration;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct SchemaBoundPolicySetRegistration {
    pub policy: String,
    pub schema: String,
}

impl ToResource<PolicyDocument> for SchemaBoundPolicySetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<PolicyDocument, Status> {
        let spec = PolicyDocumentSpec {
            active: true,
            policies: self.policy.clone(),
            schema: self.schema.clone(),
        };
        Ok(PolicyDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl Default for PolicySetRegistration {
    fn default() -> Self {
        PolicySetRegistration {
            policy: Default::default(),
        }
    }
}

impl TryFromResource<PolicyDocument> for SchemaBoundPolicySetRegistration {
    type Error = Status;

    fn try_from_resource(resource: Arc<PolicyDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl Into<PolicySetRegistration> for SchemaBoundPolicySetRegistration {
    fn into(self) -> PolicySetRegistration {
        PolicySetRegistration { policy: self.policy }
    }
}

impl ToAuditRecord for SchemaBoundPolicySetRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
