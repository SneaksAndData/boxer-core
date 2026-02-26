use std::sync::Arc;

pub trait TryFromResource<R>
where
    R: kube::Resource + Send + Sync + 'static,
{
    type Error;
    fn try_from_resource(resource: Arc<R>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
