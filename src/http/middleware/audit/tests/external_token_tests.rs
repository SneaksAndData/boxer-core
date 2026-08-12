use crate::http::middleware::audit::audit_scope::AuditScope;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::tests::MockAuditWriter;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use crate::services::audit::chained::policy_evaluation_result::PolicyEvaluationResult;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::authorization_audit_event::Reason;
use actix_web::dev::ServiceResponse;
use actix_web::web::scope;
use actix_web::{App, Error, HttpMessage, HttpRequest, HttpResponse, test, web};
use assert_matches::assert_matches;
use cedar_policy::Decision;
use std::sync::Arc;

#[actix_web::test]
async fn test_token_not_present() {
    // Arrange
    let scope = scope("").route(
        "/token",
        web::to(|| async move { actix_web::HttpResponse::Ok().finish() }),
    );
    let mut writer = MockAuditWriter::new();
    writer.expect_final_failed_event();

    let pipeline = scope.with_initial_audit_scope(Arc::new(writer));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/token").to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_external_token_message(
        response,
        "token-extraction-failed: External token not present in request extensions",
    );
}

#[actix_web::test]
async fn test_broken_token() {
    // Arrange
    let mut writer = MockAuditWriter::new();
    writer.expect_final_failed_event();

    let scope = scope("").route("/token", web::to(|| async move { HttpResponse::Ok().finish() }));
    let pipeline = scope.with_initial_audit_scope(Arc::new(writer));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "I am authorization"))
        .to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_external_token_message(
        response,
        "token-extraction-failed: Invalid header format. Expected `Bearer ...`",
    );
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
                    external_token: Some(TokenAuditEvent { token_id, }),
                    internal_token: None,
                }) => {
                    assert_eq!(token_id, format!("md5:{:x}", md5::compute("TOKEN")));

                }
            );

            HttpResponse::Ok().finish()
        }),
    );

    // Arrange
    let mut writer = MockAuditWriter::new();
    writer.expect_write().times(1).returning(|_| ());

    let pipeline = scope.with_initial_audit_scope(Arc::new(writer));
    let chain = App::new() /*.app_data(Data::new(Arc::new(writer)))*/
        .service(pipeline);
    let service = test::init_service(chain).await;

    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "Bearer TOKEN"))
        .to_request();

    // Act
    let _ = test::try_call_service(&service, request).await;

    // Assert is in the handler above
}

fn assert_external_token_message(response: anyhow::Result<ServiceResponse, Error>, message: &str) {
    assert_matches::assert_matches!(response, Err(error) => {
        let cause = error.as_error::<AuditedError>();

        assert_matches!(cause, Some(AuditedError{
            event: AuditEvent::Final(
                FinalAuditEvent{
                    external_token: Some(_),
                    internal_token: None,
                    policy_evaluation_result: PolicyEvaluationResult{ decision: Decision::Deny, reason: Some(Reason{errors: reason_errors, ..}), .. },
                    ..
                }
            ),
            ..
        }) => {
            assert!(reason_errors.contains(message), "{:?}", reason_errors)
        })
    });
}
