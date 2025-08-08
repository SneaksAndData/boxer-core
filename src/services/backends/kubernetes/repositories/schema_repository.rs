// tests module is used to test the repository
#[cfg(test)]
mod tests;

pub mod models;
mod test_reduced_schema;
#[cfg(test)]
mod test_schema;

use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::backends::kubernetes::repositories::schema_repository::models::{
    SchemaDocument, SchemaDocumentSpec,
};
use crate::services::backends::kubernetes::repositories::{
    IntoObjectRef, KubernetesRepository, SoftDeleteResource, ToResource, TryFromResource,
};
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::SchemaFragment;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use std::sync::Arc;

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

    fn to_resource_default(&self, key: &ObjectRef<SchemaDocument>) -> Result<SchemaDocument, Status> {
        let serialized = self
            .to_json_string()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(SchemaDocument {
            metadata: ObjectMeta {
                name: Some(key.name.clone()),
                namespace: key.namespace.clone(),
                ..Default::default()
            },
            spec: SchemaDocumentSpec {
                active: true,
                schema: serialized,
            },
        })
    }
}

impl IntoObjectRef<SchemaDocument> for String {
    fn into_object_ref(self, namespace: String) -> ObjectRef<SchemaDocument> {
        let mut or = ObjectRef::new(&self);
        or.namespace = Some(namespace);
        or
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

type SchemaRepository =
    dyn UpsertRepositoryWithDelete<String, SchemaFragment, DeleteError = Status, Error = Status, ReadError = Status>;
