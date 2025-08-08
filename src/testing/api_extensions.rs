use crate::services::backends::kubernetes::repositories::SoftDeleteResource;
use kube::Api;
use kube::runtime::reflector::{Lookup, ObjectRef};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::time::Duration;

#[allow(async_fn_in_trait)]
pub trait WaitForResource<T: Lookup> {
    async fn wait_for_creation(&self, name: String, namespace: String, timeout: Duration) -> T;
}

#[allow(async_fn_in_trait)]
pub trait WaitForDelete<T: SoftDeleteResource> {
    async fn wait_for_deletion<R>(&self, name: String, namespace: String, timeout: Duration) -> T;
}

impl<T> WaitForResource<T> for Api<T>
where
    T: Lookup + DeserializeOwned + Debug + Clone + Send + Sync + 'static,
    T::DynamicType: Default,
{
    async fn wait_for_creation(&self, name: String, namespace: String, timeout: Duration) -> T {
        let mut object_ref: ObjectRef<T> = ObjectRef::new(&name);
        object_ref.namespace = Some(namespace);
        let start_time = std::time::Instant::now();
        loop {
            if let Ok(Some(object)) = self.get_opt(&object_ref.name).await {
                return object;
            }
            if start_time.elapsed() > timeout {
                panic!("Timed out waiting for object creation");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl<T> WaitForDelete<T> for Api<T>
where
    T: SoftDeleteResource,
    T::DynamicType: Default,
{
    async fn wait_for_deletion<R>(&self, name: String, namespace: String, timeout: Duration) -> T {
        let mut object_ref: ObjectRef<T> = ObjectRef::new(&name);
        object_ref.namespace = Some(namespace);
        let start_time = std::time::Instant::now();
        loop {
            if let Ok(Some(object)) = self.get_opt(&object_ref.name).await {
                if object.is_deleted() {
                    return object;
                }
            }
            if start_time.elapsed() > timeout {
                panic!("Timed out waiting for object creation");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
