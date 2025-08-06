// tests module is used to test the repository
#[cfg(test)]
mod tests;

mod test_reduced_schema;
#[cfg(test)]
mod test_schema;

// Use log crate when building application
#[cfg(not(test))]
use log::warn;

// Workaround to use prinltn! for logs.
#[cfg(test)]
use std::println as warn;

// Other imports
use super::super::kubernetes_resource_manager::synchronized::SynchronizedKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use crate::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};
use anyhow::anyhow;
use async_trait::async_trait;
use cedar_policy::SchemaFragment;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::serde_json;
use kube::CustomResource;
use kube::runtime::reflector::ObjectRef;
use maplit::btreemap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

impl Default for SchemaDocument {
    fn default() -> Self {
        SchemaDocument {
            metadata: ObjectMeta::default(),
            spec: SchemaDocumentSpec::default(),
        }
    }
}

pub struct KubernetesSchemaRepository {
    resource_manger: SynchronizedKubernetesResourceManager<SchemaDocument>,
    label_selector_key: String,
    label_selector_value: String,
}

impl KubernetesSchemaRepository {
    #[allow(dead_code)] // Dead code is allowed here because this function is used in kubernetes
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manger =
            SynchronizedKubernetesResourceManager::start(config, Arc::new(LoggingUpdateHandler)).await?;
        Ok(KubernetesSchemaRepository {
            resource_manger,
            label_selector_key,
            label_selector_value,
        })
    }
}

impl Drop for KubernetesSchemaRepository {
    fn drop(&mut self) {
        if let Err(e) = self.resource_manger.stop() {
            warn!("Failed to stop KubernetesSchemaRepository: {}", e);
        }
    }
}

#[async_trait]
impl ReadOnlyRepository<String, SchemaFragment> for KubernetesSchemaRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: String) -> Result<SchemaFragment, Self::ReadError> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        let resource_object = self.resource_manger.get(or);
        let resource_object = match resource_object {
            Some(r) => r,
            None => return Err(anyhow!("Resource not found: {}", key)),
        };
        if !resource_object.spec.active {
            return Err(anyhow!("Schema is not active"));
        }
        let result: SchemaFragment = resource_object.spec.clone().try_into()?;
        Ok(result)
    }
}

#[async_trait]
impl UpsertRepository<String, SchemaFragment> for KubernetesSchemaRepository {
    type Error = anyhow::Error;

    async fn upsert(&self, key: String, entity: SchemaFragment) -> Result<(), Self::Error> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        let mut resource_ref = self.resource_manger.get(or).unwrap_or_default();
        let resource_ref = Arc::make_mut(&mut resource_ref);
        resource_ref.metadata.name = Some(key.clone());
        resource_ref.metadata.labels = Some(btreemap! {
            self.label_selector_key.clone() => self.label_selector_value.clone(),
        });
        resource_ref.metadata.namespace = Some(self.resource_manger.namespace().clone());
        resource_ref.spec.schema = entity.to_json_string()?;
        resource_ref.spec.active = true;
        self.resource_manger.replace(&key, resource_ref.clone()).await
    }

    async fn exists(&self, key: String) -> bool {
        let or: ObjectRef<SchemaDocument> =
            ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        self.resource_manger.get(or).map(|r| r.spec.active).unwrap_or(false)
    }
}

#[async_trait]
impl CanDelete<String, SchemaFragment> for KubernetesSchemaRepository {
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: String) -> Result<(), Self::DeleteError> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manger.namespace().as_str());
        let resource_ref = self.resource_manger.get(or);
        let mut resource_ref = match resource_ref {
            Some(r) => r,
            None => return Err(anyhow!("Resource not found: {}", key)),
        };
        let resource_object = Arc::make_mut(&mut resource_ref);
        resource_object.spec.active = false;
        self.resource_manger.replace(&key, resource_object.clone()).await
    }
}

impl UpsertRepositoryWithDelete<String, SchemaFragment> for KubernetesSchemaRepository {}
