#[derive(Debug)]
pub struct OwnerConflictDetails {
    pub object_name: String,
    pub object_namespace: Option<String>,
    pub current_owner: Option<String>,
}

impl OwnerConflictDetails {
    pub fn new(object_name: String, object_namespace: Option<String>) -> Self {
        OwnerConflictDetails {
            object_name,
            object_namespace,
            current_owner: None,
        }
    }

    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.current_owner = owner;
        self
    }
}
