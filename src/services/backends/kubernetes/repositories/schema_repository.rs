// tests module is used to test the repository
#[cfg(test)]
mod tests;

pub mod schema_document;
mod test_reduced_schema;
#[cfg(test)]
mod test_schema;

use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::repositories::schema_repository::schema_document::{
    SchemaDocument, SchemaDocumentSpec,
};
use crate::services::backends::kubernetes::repositories::{
    KubernetesRepository, SoftDeleteResource, ToResource, TryFromResource,
};
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::SchemaFragment;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::sync::Arc;

impl ToResource<SchemaDocument> for SchemaFragment {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<SchemaDocument, Status> {
        let serialized = self
            .to_json_string()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(SchemaDocument {
            metadata: object_meta.clone(),
            spec: SchemaDocumentSpec {
                active: true,
                schema: serialized,
            },
        })
    }
}

impl TryFromResource<SchemaDocument> for SchemaFragment {
    type Error = Status;

    fn try_into_resource(resource: Arc<SchemaDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl UpsertRepositoryWithDelete<String, SchemaFragment> for KubernetesRepository<SchemaDocument> {}

pub type SchemaRepository =
    dyn UpsertRepositoryWithDelete<String, SchemaFragment, DeleteError = Status, Error = Status, ReadError = Status>;
