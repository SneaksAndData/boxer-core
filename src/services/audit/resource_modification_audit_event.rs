use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResourceModificationAuditEvent {
    pub id: String,
    pub resource_type: String,
    pub modification_result: ModificationResult,
}

impl ResourceModificationAuditEvent {
    pub fn new(id: String, resource_type: String, modification_result: ModificationResult) -> Self {
        Self {
            id,
            resource_type,
            modification_result,
        }
    }
}

#[derive(Serialize)]
pub enum ModificationResult {
    Success(String),
    Failure,
}

impl<T, E> From<&Result<T, E>> for ModificationResult
where
    T: ToAuditRecord,
{
    fn from(result: &Result<T, E>) -> Self {
        match result {
            Ok(value) => ModificationResult::Success(value.to_audit_record()),
            Err(_) => ModificationResult::Failure,
        }
    }
}
