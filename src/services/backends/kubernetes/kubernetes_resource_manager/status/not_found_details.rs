use kube::runtime::reflector::ObjectRef;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct NotFoundDetails {
    pub name: String,
    pub namespace: Option<String>,
    pub resource_type: String,
}

impl NotFoundDetails {
    pub fn new(name: String, namespace: Option<String>, resource_type: String) -> Self {
        NotFoundDetails {
            name,
            namespace,
            resource_type,
        }
    }
}

impl Display for NotFoundDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let namespace = self.namespace.as_deref().unwrap_or("unknown");
        write!(
            f,
            "Resource of kind '{}' with name: '{}',  namespace '{}' not found",
            self.resource_type, self.name, namespace
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
            resource_type: R::kind(&object_ref.dyntype).to_string(),
        }
    }
}
