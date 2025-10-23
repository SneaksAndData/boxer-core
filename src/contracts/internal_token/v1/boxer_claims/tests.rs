use super::*;
use crate::contracts::dynamic_claims_collection::DynamicClaims;
use crate::contracts::internal_token::v1::{PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, VALIDATOR_SCHEMA_ID_KEY};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::collections::HashMap;

#[test]
fn test_to_boxer_claims_success() {
    let mc = MockClaims::base();
    let claims = mc.to_boxer_claims().expect("should succeed");
    assert!(claims.schema.to_json_string().unwrap().contains("entityTypes"));
    assert_eq!(claims.schema_id, "schema-v1");
    assert_eq!(claims.validator_schema_id, "validator-schema-v1");
    assert_eq!(claims.principal.uid().to_string(), "User::\"alice\"");
}

#[test]
fn test_missing_schema() {
    let mc = MockClaims::base().remove_value(SCHEMA_KEY);
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Missing schema"));
}

#[test]
fn test_missing_principal() {
    let mc = MockClaims::base().remove_value(PRINCIPAL_KEY);
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Missing schema"));
}

#[test]
fn test_missing_schema_id() {
    let mc = MockClaims::base().remove_claim(SCHEMA_ID_KEY);
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Missing schema_id"));
}

#[test]
fn test_missing_validator_schema_id() {
    let mc = MockClaims::base().remove_claim(VALIDATOR_SCHEMA_ID_KEY);
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Missing schema_id"));
}

#[test]
fn test_invalid_schema() {
    let mc = MockClaims::base().with_value(SCHEMA_KEY, json!("not-an-object"));
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Invalid schema"));
}

#[test]
fn test_invalid_principal() {
    let mc = MockClaims::base().with_value(PRINCIPAL_KEY, json!({"bad": "format"}));
    let err = mc.to_boxer_claims().unwrap_err();
    assert!(err.to_string().contains("Invalid principal"));
}
struct MockClaims {
    values: HashMap<&'static str, Value>,
    claims: HashMap<&'static str, String>,
}

impl MockClaims {
    fn base() -> Self {
        let mut values = HashMap::new();
        // Minimal valid schema fragment (entity types map)
        values.insert(
            SCHEMA_KEY,
            json!({
                "PhotoApp": {
                    "entityTypes": {
                        "User": {},
                        "Photo": {}
                    },
                    "actions": { }
                }
            }),
        );
        // Minimal valid principal entity
        values.insert(
            PRINCIPAL_KEY,
            json!({
                "uid": { "type": "User", "id": "alice" },
                "attrs": {},
                "parents": []
            }),
        );

        let mut claims = HashMap::new();
        claims.insert(SCHEMA_ID_KEY, "schema-v1".to_string());
        claims.insert(VALIDATOR_SCHEMA_ID_KEY, "validator-schema-v1".to_string());

        Self { values, claims }
    }

    fn with_value(mut self, k: &'static str, v: Value) -> Self {
        self.values.insert(k, v);
        self
    }

    fn remove_value(mut self, k: &'static str) -> Self {
        self.values.remove(k);
        self
    }

    fn remove_claim(mut self, k: &'static str) -> Self {
        self.claims.remove(k);
        self
    }
}

impl DynamicClaims for MockClaims {
    fn get_claim(&self, key: &str) -> Option<String> {
        self.claims.get(key).cloned()
    }
    fn get_value(&self, key: &str) -> Option<Value> {
        self.values.get(key).as_deref().cloned()
    }
}
