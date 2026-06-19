use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::initial_audit_scope;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationResult;
use actix_web::web::scope;
use actix_web::{App, HttpMessage, HttpRequest, test, web};
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
    writer
        .expect_write()
        .times(1)
        .withf(|event| {
            matches!(
                event,
                AuditEvent::Final(ChainedAuditEvent {
                    external_token: Some(TokenAuditEvent {
                        token_id: None,
                        result: Some(TokenValidationResult::Deny),
                        reason_errors: _,
                        token_type: None,
                    }),
                    internal_token: None,
                    action: None,
                    actor: None,
                    resource: None,
                    decision: Some(Decision::Deny),
                    reason: None,
                })
            )
        })
        .returning(|_| ());

    let pipeline = initial_audit_scope(scope, Arc::new(writer));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/token").to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_matches!(response, Err(error) => {
        let cause = error.as_error::<AuditedError>();

        assert_matches!(cause, Some(AuditedError{
            event: AuditEvent::Final(
                ChainedAuditEvent{
                    external_token: Some(TokenAuditEvent{
                        token_id: None,
                        result: Some(TokenValidationResult::Deny),
                        reason_errors,
                        token_type: None
                    }),
                    internal_token: None,
                    action: None,
                    actor: None,
                    resource: None,
                    decision: Some(Decision::Deny),
                    reason: None
                }
            ),
            ..
        }) => {
            assert!(reason_errors.contains("token-not-present"), "{:?}", reason_errors)
        })
    });
}

#[actix_web::test]
async fn test_broken_token() {
    // Arrange
    let scope = scope("").route(
        "/token",
        web::to(|| async move { actix_web::HttpResponse::Ok().finish() }),
    );
    let mut writer = MockAuditWriter::new();
    writer
        .expect_write()
        .times(1)
        .withf(|event| {
            matches!(
                event,
                AuditEvent::Final(ChainedAuditEvent {
                    external_token: Some(TokenAuditEvent {
                        token_id: None,
                        result: Some(TokenValidationResult::Deny),
                        reason_errors: _,
                        token_type: None
                    }),
                    internal_token: None,
                    action: None,
                    actor: None,
                    resource: None,
                    decision: Some(Decision::Deny),
                    reason: None
                })
            )
        })
        .returning(|_| ());

    let pipeline = initial_audit_scope(scope, Arc::new(writer));

    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "I am authorization"))
        .to_request();

    // Act
    let response = test::try_call_service(&service, request).await;

    // Assert that the error in the result has the required structure
    assert_matches!(response, Err(error) => {
        let cause = error.as_error::<AuditedError>();

        assert_matches!(cause, Some(AuditedError{
            event: AuditEvent::Final(
                ChainedAuditEvent{
                    external_token: Some(TokenAuditEvent{
                        token_id: None,
                        result: Some(TokenValidationResult::Deny),
                        reason_errors,
                        token_type: None
                    }),
                    internal_token: None,
                    action: None,
                    actor: None,
                    resource: None,
                    decision: Some(Decision::Deny),
                    reason: None
                }
            ),
            ..
        }) => {
            assert!(reason_errors.contains("token-extraction-failed: Invalid token format"), "{:?}", reason_errors)
        })
    });
}

#[actix_web::test]
async fn test_successful_token() {
    let scope = scope("").route(
        "/token",
        web::to(|request: HttpRequest| async move {
            let event = request.extensions().get::<AuditEvent>().unwrap().clone();
            actix_web::HttpResponse::Ok().finish()
        }),
    );
    let mut writer = MockAuditWriter::new();
    writer.expect_write().times(1).returning(|_| ());

    let pipeline = initial_audit_scope(scope, Arc::new(writer));
    let chain = App::new().service(pipeline);
    let service = test::init_service(chain).await;
    let request = test::TestRequest::get()
        .uri("/token")
        .append_header(("Authorization", "Bearer token"))
        .to_request();
    let response = test::try_call_service(&service, request).await;
    assert_matches!(response, Ok(_) => {})
}

mock! {

    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }
}

/*

    // Arrange
    let ctx = MockAuditContext::try_create_audit_context_context();

    // Set expectations on the static creation method
    ctx.expect().once().returning_st(|request| {
        let mut mock_instance = MockAuditContext::new();

        // Set expectations on the instance methods (like `Into<ServiceRequest>`)
        mock_instance.expect_into().once().return_once_st(move || request);

        Ok(mock_instance)
    });

    let chain = App::new()
        .wrap(from_fn(begin_audit_chain::<MockAuditContext>)) // Called first
        .default_service(web::to(|| async move { HttpResponse::Ok().finish() }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act
    let _ = test::call_service(&service, request).await;

    // Expectations are verified automatically when `mock_audit` is dropped at the end of the scope.
}


 */
