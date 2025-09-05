pub trait ToAuditRecord {
    fn to_audit_record(&self) -> String;
}

impl ToAuditRecord for (String, String) {
    fn to_audit_record(&self) -> String {
        format!("Schema: {}, id: {}", self.0, self.1)
    }
}

impl ToAuditRecord for String {
    fn to_audit_record(&self) -> String {
        self.clone()
    }
}
