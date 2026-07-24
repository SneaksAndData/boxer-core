use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;
use crate::http::middleware::audit::audit_scope::AuditScope;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::tests::MockAuditWriter;
use crate::http::middleware::token_decryptor_middleware::decryptor::Decryptor;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use actix_web::dev::ServiceResponse;
use actix_web::web::scope;
use actix_web::{App, Error, test, web};
use assert_matches::assert_matches;
use cedar_policy::Decision;
use mockall::mock;
use std::sync::Arc;

#[actix_web::test]
async fn test_token_not_present() {
    // Arrange
    let scope = scope("").route(
        "/token",
        web::to(|| async move { actix_web::HttpResponse::Ok().finish() }),
    );
    let mut writer = MockAuditWriter::new();
    writer.expect_final_failed_internal_token_event();

    let pipeline = scope.continue_audit_scope(Arc::new(writer), Arc::new(MockDecryptor::new()));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/token").to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_internal_token_message(
        response,
        "token-extraction-failed: Internal token not present in request extensions",
    );
}

fn assert_internal_token_message(response: anyhow::Result<ServiceResponse, Error>, message: &str) {
    assert_matches::assert_matches!(response, Err(error) => {
        let cause = error.as_error::<AuditedError>();

        assert_matches!(cause, Some(AuditedError{
            event: AuditEvent::Final(
                FinalAuditEvent{
                    internal_token: Some(TokenAuditEvent{
                        result: Some(TokenValidationResult::Deny),
                        reason_errors,
                        ..
                    }),
                    policy_evaluation_result: PolicyEvaluationResult{ decision: Decision::Deny, .. },
                    ..
                }
            ),
            ..
        }) => {
            assert!(reason_errors.contains(message), "{:?}", reason_errors)
        })
    });
}

mock! {
    pub Decryptor {}

    impl Decryptor for Decryptor {
        fn decrypt(&self, token: EncryptedToken) -> Result<DynamicClaimsCollection, anyhow::Error>;
    }

}
