use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::API_VERSION_KEY;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::contracts::internal_token::v2::{
    AUDIT_EVENT, PRINCIPAL_KEY, SCHEMA_ID_KEY, SCHEMA_KEY, VALIDATOR_SCHEMA_ID_KEY,
};
use crate::http::middleware::audit::audit_scope::AuditScope;
use crate::http::middleware::audit::tests::MockAuditWriter;
use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::token_decryption_service::TokenDecryptionService;
use crate::services::token_decryption_service::encryption_keys::EncryptionKeys;
use crate::services::token_decryption_service::token_settings::TokenValidationSettings;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::web::scope;
use actix_web::{App, Error, HttpMessage, HttpRequest, HttpResponse, test, web};
use mockall::mock;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[actix_web::test]
async fn test_token_not_present() {
    // Arrange
    let scope = scope("").route(
        "/token",
        web::to(|| async move { actix_web::HttpResponse::Ok().finish() }),
    );
    let mut writer = MockAuditWriter::new();
    writer.expect_final_failed_internal_token_event("Internal token not present in request extensions".to_string());

    let pipeline = scope.continue_audit_scope(Arc::new(writer), Arc::new(MockDecryptor::new()));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/token").to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_internal_token_message(response, "Internal token not present in request extensions");
}

#[actix_web::test]
async fn test_broken_token_format() {
    // Arrange
    let mut writer = MockAuditWriter::new();
    writer.expect_final_failed_internal_token_event("Invalid header format. Expected `Bearer ...`".to_string());

    let scope = scope("").route("/token", web::to(|| async move { HttpResponse::Ok().finish() }));
    let pipeline = scope.continue_audit_scope(Arc::new(writer), Arc::new(MockDecryptor::new()));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "I am authorization"))
        .to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_internal_token_message(response, "Invalid header format. Expected `Bearer ...`");
}

#[actix_web::test]
async fn test_incorrect_token_decoder() {
    // Arrange
    let mut writer = MockAuditWriter::new();
    // delivers the AuditEvent into the token controller.
    writer.expect_write().returning(|_| ());

    let scope = scope("").route("/token", web::to(|| async move { HttpResponse::Ok().finish() }));
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

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "Bearer I-am-not-a-valid-token"))
        .to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_internal_token_message(response, "Invalid JWT format: The input cannot be recognized as a JWT.");
}

#[actix_web::test]
async fn test_successful_token() {
    let scope = scope("").route(
        "/token",
        web::to(|request: HttpRequest| async move {
            // Assert that the intermediate audit event has the expected structure and values

            let event = request.extensions().get::<AuditEvent>().unwrap().clone();
            assert_matches::assert_matches!(
                event,
                AuditEvent::Intermediate(IntermediateAuditEvent{
                    internal_token: Some(TokenAuditEvent { token_id, }),
                    external_token: Some(TokenAuditEvent { token_id: external_token_id, .. }),
                }) => {
                    assert_eq!(token_id, format!("md5:{:x}", md5::compute("TOKEN")));
                    assert_eq!(external_token_id, "token-id");

                }
            );

            HttpResponse::Ok().finish()
        }),
    );

    // Arrange
    let mut writer = MockAuditWriter::new();

    // Not checking  the expected value here since this test only how does the pipeline
    // delivers the AuditEvent into the token controller.
    writer.expect_write().returning(|_| ());

    let mut decryptor = MockDecryptor::new();
    decryptor.expect_decrypt().returning(|_| {
        let mut dcc = DynamicClaimsCollection::new();
        dcc.set_claim(API_VERSION_KEY, Some("v2".into())).unwrap();
        dcc.set_claim("aud", Some("boxer".into())).unwrap();
        dcc.set_claim("iss", Some("issuer".into())).unwrap();
        dcc.set_claim(
            SCHEMA_KEY,
            Some(json!({
                "PhotoApp": {
                    "entityTypes": {
                        "User": {},
                        "Photo": {}
                    },
                    "actions": {}
                }
            })),
        )
        .unwrap();
        dcc.set_claim(SCHEMA_ID_KEY, Some("schema_id".into())).unwrap();
        dcc.set_claim(VALIDATOR_SCHEMA_ID_KEY, Some("validator_schema_id".into()))
            .unwrap();
        dcc.set_claim(
            PRINCIPAL_KEY,
            Some(json!({
                "uid": { "type": "User", "id": "alice" },
                "attrs": {},
                "parents": []
            })),
        )
        .unwrap();
        dcc.set_claim(
            AUDIT_EVENT,
            Some(serde_json::to_value(TokenAuditEvent::external("token-id".to_string())).unwrap()),
        )
        .unwrap();
        Ok(dcc)
    });

    let pipeline = scope.continue_audit_scope(Arc::new(writer), Arc::new(decryptor));
    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;

    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "Bearer TOKEN"))
        .to_request();

    // Act
    let _ = test::try_call_service(&service, request).await;

    // Assert is in the handler above
}

fn assert_internal_token_message(response: anyhow::Result<ServiceResponse, Error>, message: &str) {
    let error = response.expect_err("Token failures must remain service errors");
    assert_eq!(error.as_response_error().status_code(), StatusCode::UNAUTHORIZED);
    let response = error.error_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response.into_body().try_into_bytes().unwrap();
    assert!(body.is_empty(), "Token error details must not be exposed: {message}");
}

mock! {
    pub Decryptor {}

    impl Decryptor for Decryptor {
        fn decrypt(&self, token: EncryptedToken) -> Result<DynamicClaimsCollection, anyhow::Error>;
    }

}
