use k8s_openapi::NamespaceResourceScope;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait SoftDeleteResource:
    kube::Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
    fn is_deleted(&self) -> bool;
    fn set_deleted(&mut self);
    fn clear_managed_fields(&mut self);
}
