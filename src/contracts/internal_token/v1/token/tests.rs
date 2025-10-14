use crate::contracts::internal_token::v1::boxer_claims::ToBoxerClaims;
use crate::contracts::internal_token::v1::token::InternalToken;
use cedar_policy::{Entity, EntityUid, SchemaFragment};
use josekit::jwt::JwtPayload;
use serde_json::json;
use std::time::Duration;

#[test]
fn test_serialization_integrity() {
    let token = InternalToken::new(
        make_principal(),
        make_schema(),
        "alice-ext".to_string(),
        "github".to_string(),
        "schema-v1".to_string(),
        Duration::from_secs(600),
        "validator-schema-v1".to_string(),
    );

    let jwt: JwtPayload = token.try_into().expect("to jwt");
    let boxer_claims = jwt.to_boxer_claims().expect("jwt to boxer claims");

    assert_eq!(boxer_claims.schema_id, "schema-v1");
    assert_eq!(boxer_claims.validator_schema_id, "validator-schema-v1");
    assert_eq!(boxer_claims.principal.uid().to_string(), "User::\"alice\"");
    assert!(
        boxer_claims
            .schema
            .to_json_value()
            .unwrap()
            .get("PhotoApp")
            .and_then(|n| n.get("entityTypes"))
            .and_then(|et| et.get("User"))
            .is_some()
    );
}

fn make_principal() -> Entity {
    let uid: EntityUid = r#"User::"alice""#.parse().unwrap();
    Entity::new(uid, Default::default(), Default::default()).expect("to be valid")
}

fn make_schema() -> SchemaFragment {
    let schema_json = json!({
        "PhotoApp": {
            "entityTypes": {
                "User": {},
                "Photo": {}
            },
            "actions": { }
        }
    });
    SchemaFragment::from_json_value(schema_json).unwrap()
}
