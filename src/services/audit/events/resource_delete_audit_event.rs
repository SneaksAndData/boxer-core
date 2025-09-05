use serde::Serialize;

#[derive(Serialize)]
pub struct ResourceDeleteAuditEvent {
    pub id: String,
    pub resource_type: String,
    pub successful: bool,
}

impl ResourceDeleteAuditEvent {
    pub fn new(id: String, resource_type: String, successful: bool) -> Self {
        Self {
            id,
            resource_type,
            successful,
        }
    }
}
