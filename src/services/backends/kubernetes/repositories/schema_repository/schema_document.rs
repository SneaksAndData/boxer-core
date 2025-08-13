use crate::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use crate::services::backends::kubernetes::repositories::SoftDeleteResource;
use anyhow::anyhow;
use cedar_policy::SchemaFragment;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "SchemaDocument",
    plural = "schemas",
    singular = "schema",
    namespaced
)]
pub struct SchemaDocumentSpec {
    pub schema: String,
    pub active: bool,
}

impl UpdateLabels for SchemaDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl TryInto<SchemaFragment> for SchemaDocumentSpec {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<SchemaFragment, Self::Error> {
        SchemaFragment::from_json_str(self.schema.as_str()).map_err(|err| anyhow!("{}", err))
    }
}

impl Default for SchemaDocument {
    fn default() -> Self {
        SchemaDocument {
            metadata: ObjectMeta::default(),
            spec: SchemaDocumentSpec::default(),
        }
    }
}

impl SoftDeleteResource for SchemaDocument {
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
