use crate::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
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

impl TryFrom<SchemaFragment> for SchemaDocumentSpec {
    type Error = anyhow::Error;

    fn try_from(schema: SchemaFragment) -> Result<Self, Self::Error> {
        let serialized = schema
            .to_json_value()
            .map_err(|err| anyhow!("Failed to convert schema to JSON string: {}", err))?;
        Ok(SchemaDocumentSpec {
            active: true,
            schema: serde_json::to_string_pretty(&serialized)?,
        })
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
