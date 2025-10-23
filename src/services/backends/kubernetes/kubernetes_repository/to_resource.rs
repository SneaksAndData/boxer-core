use crate::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;

pub trait ToResource<R>
where
    R: SoftDeleteResource,
{
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<R, Status>;
    fn to_resource_default(&self, object_ref: &ObjectRef<R>) -> Result<R, Status> {
        let object_meta = ObjectMeta {
            name: Some(object_ref.name.clone()),
            namespace: object_ref.namespace.clone(),
            ..Default::default()
        };
        self.to_resource(&object_meta)
    }
}
