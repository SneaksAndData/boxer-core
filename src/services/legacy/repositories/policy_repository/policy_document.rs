use crate::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use crate::services::legacy::repositories::policy_repository::schema_bound_policy_set_registration::SchemaBoundPolicySetRegistration;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "PolicyDocument",
    plural = "policy-documents",
    singular = "policy-document",
    namespaced
)]
pub struct PolicyDocumentSpec {
    pub active: bool,
    pub policies: String,
    pub schema: String,
}

impl Default for PolicyDocument {
    fn default() -> Self {
        PolicyDocument {
            metadata: ObjectMeta {
                name: None,
                namespace: None,
                ..Default::default()
            },
            spec: PolicyDocumentSpec {
                active: true,
                policies: String::new(),
                schema: Default::default(),
            },
        }
    }
}

impl From<SchemaBoundPolicySetRegistration> for PolicyDocumentSpec {
    fn from(value: SchemaBoundPolicySetRegistration) -> Self {
        PolicyDocumentSpec {
            active: true,
            policies: value.policy.to_string(),
            schema: value.schema.to_string(),
        }
    }
}

impl Into<SchemaBoundPolicySetRegistration> for PolicyDocumentSpec {
    fn into(self) -> SchemaBoundPolicySetRegistration {
        SchemaBoundPolicySetRegistration {
            policy: self.policies,
            schema: self.schema,
        }
    }
}

impl SoftDeleteResource for PolicyDocument {
    fn is_deleted(&self) -> bool {
        !self.spec.active
    }

    fn set_deleted(&mut self) {
        self.spec.active = false;
    }

    fn clear_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}

impl UpdateLabels for PolicyDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}
