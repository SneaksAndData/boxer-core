mod external_token_tests;
mod internal_token_tests;

use crate::contracts::internal_token::v2::boxer_claims::BoxerClaims;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audit_scope::AuditScope;
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::encrypted_token_service::EncryptedTokenService;
use crate::services::external_identity_validator::external_identity::ExternalIdentity;
use crate::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use crate::services::token_decryption_service::encryption_keys::EncryptionKeys;
use crate::services::token_decryption_service::token_settings::TokenValidationSettings;
use crate::services::token_decryption_service::TokenDecryptionService;
use crate::services::token_service::internal_token_service::token_provider::principal::Principal;
use crate::services::token_service::internal_token_service::token_provider::principal_service::PrincipalService;
use crate::services::token_service::internal_token_service::token_provider::TokenProvider;
use crate::services::validation_service::cedar_validation_service::CedarValidationService;
use crate::services::validation_service::path_segment::PathSegment;
use crate::services::validation_service::request_context::RequestContext;
use crate::services::validation_service::request_segment::RequestSegment;
use crate::services::validation_service::schema_provider::SchemaProvider;
use crate::services::validation_service::ValidationService;
use actix_web::http::StatusCode;
use actix_web::web::{scope, ReqData};
use actix_web::{test, web, App, HttpMessage, HttpRequest, HttpResponse};
use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::PolicySet;
use cedar_policy::SchemaFragment;
use cedar_policy::{Decision, Policy};
use cedar_policy::{Entity, EntityUid, Schema};
use mockall::mock;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// Integration tests that validates the issuance of the token version 1.
/// This test tests happy path and includes both external and internal HTTP pipelines.
#[actix_web::test]
async fn test_token_v1() {
    // Arrange
    let mut mock_principal_service = MockPrincipalService::new();
    let principal = Principal::new(make_principal_entity(), "schema-v1".into());
    let schema = make_schema_fragment();

    mock_principal_service
        .expect_get_principal()
        .returning(move |_| Ok(principal.clone()));
    mock_principal_service
        .expect_get_schemas()
        .returning(move |_| Ok(schema.clone()));
    mock_principal_service
        .expect_get_validator_schema()
        .returning(|_| Ok("validator-schema-v1".into()));

    let token_service = EncryptedTokenService::new(
        Arc::new(mock_principal_service),
        Arc::new("0123456789ABCDEF0123456789ABCDEF".into()),
        "key-id".into(),
        "example.com".into(),
        "example.com".into(),
        "A128CBC-HS256".into(),
        MetricsProvider::new("tests", "tests".into()),
    );

    let token = token_service
        .issue_token(ExternalIdentity::for_test("user-id", "identity-provider"))
        .await
        .unwrap();

    let mut mock_schema_provider = MockSchemaProvider::new();
    mock_schema_provider
        .expect_get_schema()
        .returning(|_| Ok(Schema::from_json_value(json!({})).unwrap()));

    let mut mock_action_repository = MockActionRepository::new();
    mock_action_repository
        .expect_get()
        .returning(|_| Ok(r#"Action::"post""#.parse().unwrap()));

    let mut mock_resource_repository = MockResourceRepository::new();
    mock_resource_repository
        .expect_get()
        .returning(|_| Ok(r#"Http::"example.com""#.parse().unwrap()));

    let mut mock_policy_repository = MockPolicyRepository::new();
    mock_policy_repository.expect_get().returning(|_| {
        Ok(PolicySet::from_policies(Policy::from_str(
            r#"
                    permit(principal, action, resource);
                   "#,
        ))
        .unwrap())
    });

    let validation_service = Arc::new(CedarValidationService::<BoxerClaims>::new(
        Arc::new(mock_schema_provider),
        Arc::new(mock_action_repository),
        Arc::new(mock_resource_repository),
        Arc::new(mock_policy_repository),
        MetricsProvider::new("tests", "tests".into()),
    ));

    let scope = scope("").route(
        "/token",
        web::to({
            move |boxer_claims: ReqData<BoxerClaims>, request_context: RequestContext, http_request: HttpRequest| {
                let validation_service = validation_service.clone();
                async move {
                    let mut extensions = http_request.extensions_mut();
                    let event = extensions.get_mut::<AuditEvent>().unwrap();
                    let response = validation_service
                        .validate(boxer_claims.into_inner(), request_context, event)
                        .await;
                    HttpResponse::build(response.map(|_| StatusCode::OK).unwrap_or(StatusCode::FORBIDDEN)).finish()
                }
            }
        }),
    );
    let mut writer = MockAuditWriter::new();
    writer.expect_final_success_event();
    let mut keys = HashMap::default();
    keys.insert("key-id".into(), "0123456789ABCDEF0123456789ABCDEF".into());
    let encryption_keys = EncryptionKeys::new(keys);
    let token_validation_settings = TokenValidationSettings {
        audience: "example.com".into(),
        issuer: "example.com".into(),
        keys: "".into(),
    };
    let decryptor = TokenDecryptionService::new(encryption_keys, token_validation_settings);
    let pipeline = scope.continue_audit_scope(Arc::new(writer), Arc::new(decryptor));
    let webapp = App::new().service(pipeline);
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", format!("Bearer {}", token)))
        .append_header(("X-Original-URL", "http://example.com"))
        .append_header(("X-Original-Method", "POST"))
        .to_request();
    let service = test::init_service(webapp).await;

    // Act
    let response = test::try_call_service(&service, request).await;
    println!("response: {:?}", response);
    assert_eq!(response.unwrap().status(), 200);
}

fn make_principal_entity() -> Entity {
    let uid: EntityUid = r#"User::"alice""#.parse().unwrap();
    Entity::new(uid, Default::default(), Default::default()).expect("to be valid")
}

fn make_schema_fragment() -> SchemaFragment {
    let schema_json = json!({
        "PhotoApp": {
            "entityTypes": {
                "User": {},
                "Photo": {}
            },
            "actions": {}
        }
    });
    SchemaFragment::from_json_value(schema_json).unwrap()
}

mock! {
    pub PrincipalService {}

    #[async_trait]
    impl PrincipalService for PrincipalService {
        async fn get_principal(&self, external_identity: ExternalIdentity) -> Result<Principal>;
        async fn get_validator_schema(&self, external_identity: ExternalIdentity) -> Result<String>;
        async fn get_schemas(&self, schema_id: String) -> Result<SchemaFragment, anyhow::Error>;
    }
}

mock! {

    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }
}

impl MockAuditWriter {
    fn expect_final_failed_event(&mut self) -> () {
        self.expect_write()
            .times(1)
            .withf(|event| {
                matches!(
                    event,
                    AuditEvent::Final(FinalAuditEvent {
                        external_token: Some(TokenAuditEvent {
                            result: Some(TokenValidationResult::Deny),
                            reason_errors: _,
                            ..
                        }),
                        internal_token: None,
                        policy_evaluation_result: PolicyEvaluationResult {
                            action: None,
                            actor: None,
                            resource: None,
                            decision: Decision::Deny,
                            reason: None
                        }
                    })
                )
            })
            .returning(|_| ());
    }

    fn expect_final_success_event(&mut self) -> () {
        self.expect_write()
            .times(1)
            .withf(|event| {
                matches!(
                    event,
                    AuditEvent::Final(FinalAuditEvent{
                        external_token: Some(TokenAuditEvent {
                            token_id: _,
                            result: _,
                            reason_errors: _,
                        }),
                        internal_token: Some(TokenAuditEvent {
                            token_id: _,
                            result: _,
                            reason_errors: _,
                        }),
                        policy_evaluation_result: PolicyEvaluationResult {
                            action: Some(action),
                            actor: Some(actor),
                            resource: Some(resource),
                            reason: Some(reason),
                            decision: Decision::Allow,
                        }
                    }) if action == r#"Action::"post""#
                        && actor == r#"User::"alice""#
                        && resource == r#"Http::"example.com""#
                        && reason.policies.contains("policy0")
                        && reason.errors.is_empty()
                )
            })
            .returning(|_| ());
    }

    fn expect_final_failed_internal_token_event(&mut self) -> () {
        self.expect_write()
            .times(1)
            .withf(|event| {
                matches!(
                    event,
                    AuditEvent::Final(FinalAuditEvent {
                        external_token: None,
                        internal_token: Some(TokenAuditEvent {
                            result: Some(TokenValidationResult::Deny),
                            reason_errors,
                            ..
                        }),
                        policy_evaluation_result: PolicyEvaluationResult {
                            action: None,
                            actor: None,
                            resource: None,
                            decision: Decision::Deny,
                            reason: None
                        }
                    }) if reason_errors.contains("token-extraction-failed: Internal token not present in request extensions")
                )
            })
            .returning(|_| ());
    }
}

mock! {
    pub SchemaProvider {}

    #[async_trait]
    impl SchemaProvider<BoxerClaims> for SchemaProvider {
        async fn get_schema(&self, claims: &BoxerClaims) -> Result<Schema>;
    }
}

mock! {
    pub ActionRepository {}

    #[async_trait]
    impl ReadOnlyRepository<(String, Vec<RequestSegment>), EntityUid> for ActionRepository {
        type ReadError = anyhow::Error;
        async fn get(&self, key: (String, Vec<RequestSegment>)) -> Result<EntityUid, anyhow::Error>;
    }
}

mock! {
    pub ResourceRepository {}

    #[async_trait]
    impl ReadOnlyRepository<(String, Vec<PathSegment>), EntityUid> for ResourceRepository {
        type ReadError = anyhow::Error;
        async fn get(&self, key: (String, Vec<PathSegment>)) -> Result<EntityUid, anyhow::Error>;
    }
}

mock! {
    pub PolicyRepository {}

    #[async_trait]
    impl ReadOnlyRepository<String, PolicySet> for PolicyRepository {
        type ReadError = anyhow::Error;
        async fn get(&self, key: String) -> Result<PolicySet, anyhow::Error>;
    }
}
