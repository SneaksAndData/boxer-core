use kube::runtime::reflector::ObjectRef;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct OwnerConflictDetails {
    pub name: String,
    pub namespace: Option<String>,
    pub current_owner: Option<String>,
    pub resource_type: String,
}

impl OwnerConflictDetails {
    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.current_owner = owner;
        self
    }
}

impl<R> From<&ObjectRef<R>> for OwnerConflictDetails
where
    R: kube::Resource,
{
    fn from(object_ref: &ObjectRef<R>) -> Self {
        OwnerConflictDetails {
            name: object_ref.name.clone(),
            namespace: object_ref.namespace.clone(),
            resource_type: R::kind(&object_ref.dyntype).to_string(),
            current_owner: None,
        }
    }
}

impl Display for OwnerConflictDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource of kind '{}' with name: '{}',  namespace '{}' is not owned by us, current owner: {}",
            self.resource_type,
            self.name,
            self.namespace.as_deref().unwrap_or("unknown"),
            self.current_owner.as_deref().unwrap_or("unknown")
        )
    }
}
