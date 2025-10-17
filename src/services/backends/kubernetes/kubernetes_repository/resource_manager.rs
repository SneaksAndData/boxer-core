use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use async_trait::async_trait;
use k8s_openapi::NamespaceResourceScope;
use kube::Resource;
use kube::runtime::reflector::ObjectRef;
use std::sync::Arc;

#[async_trait]
pub trait ResourceManager<R>
where
    R: Resource<Scope = NamespaceResourceScope>,
{
    async fn get_uncached(&self, object_ref: &ObjectRef<R>) -> Result<R, Status>;
    async fn upsert(&self, object_ref: &ObjectRef<R>, resource: R) -> Result<R, Status>;
    fn get(&self, object_ref: &ObjectRef<R>) -> anyhow::Result<Arc<R>, Status>;

    fn namespace(&self) -> String;
}
