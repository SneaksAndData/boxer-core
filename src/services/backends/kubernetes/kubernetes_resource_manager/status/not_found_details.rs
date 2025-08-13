use kube::runtime::reflector::ObjectRef;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct NotFoundDetails {
    pub name: String,
    pub namespace: Option<String>,
}

impl NotFoundDetails {
    pub fn new(name: String, namespace: Option<String>) -> Self {
        NotFoundDetails { name, namespace }
    }
}

impl Display for NotFoundDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let namespace = self.namespace.as_deref().unwrap_or("unknown");
        write!(
            f,
            "Resource name: '{}',  namespace '{}' not found",
            self.name, namespace
        )
    }
}

impl<R> From<&ObjectRef<R>> for NotFoundDetails
where
    R: kube::Resource,
{
    fn from(object_ref: &ObjectRef<R>) -> Self {
        NotFoundDetails {
            name: object_ref.name.clone(),
            namespace: object_ref.namespace.clone(),
        }
    }
}
