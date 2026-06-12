use crate::http::middleware::audit::audited_error::AuditedError;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::chained::token_audit_event::TokenAuditEvent;
use actix_web::error::{ErrorInternalServerError, InternalError};
use actix_web::http::StatusCode;
use actix_web::test::TestRequest;
use actix_web::{HttpMessage, HttpResponse};
use anyhow::anyhow;
use assert_matches::assert_matches;
use pretty_assertions::assert_eq;

#[test]
#[should_panic(expected = "Attempt to wrap an error without an audit event")]
fn test_audited_error_wrap_panic() {
    // Arrange

    // Act
    AuditedError::wrap(InternalError::new(anyhow!("Error"), StatusCode::INTERNAL_SERVER_ERROR));

    // Assert
    // Test should panic
}

#[test]
fn test_audited_error_wrap_success() {
    // Arrange
    let mut response = HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    response
        .extensions_mut()
        .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));

    // Act
    let result = AuditedError::wrap(InternalError::from_response(anyhow!("Error"), response));

    // Assert
    assert_matches!(result, audited_error => {
        assert_matches!(audited_error.event, AuditEvent::Intermediate(_));
        assert_eq!(audited_error.cause.to_string(), "Error");
    });
}

#[test]
#[should_panic(expected = "Attempt to wrap an error without an audit event")]
fn test_audited_error_from_request_no_audit_event() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();

    // Act
    AuditedError::from_request(&request, ErrorInternalServerError("Some error"));

    // Assert
    // Test should panic
}

#[test]
#[should_panic(expected = "Final audit event in a request should not be wrapped for an error")]
fn test_audited_error_from_request_final_audit_event() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();
    request
        .request()
        .extensions_mut()
        .insert(AuditEvent::Final(ChainedAuditEvent::empty()));

    // Act
    AuditedError::from_request(&request, ErrorInternalServerError("Some error"));

    // Assert
    // Test should panic
}

#[test]
fn test_audited_error_from_request_success() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();
    request
        .request()
        .extensions_mut()
        .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));

    // Act
    let error = AuditedError::from_request(&request, ErrorInternalServerError("Error"));

    // Assert
    assert_matches!(error, audited_error => {
        assert_matches!(audited_error.event, AuditEvent::Intermediate(_));
        assert_eq!(audited_error.cause.to_string(), "Error");
    });
}

#[test]
fn test_audited_error_external_token_not_present() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();
    request
        .request()
        .extensions_mut()
        .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));

    // Act
    let error = AuditedError::external_token_not_present(&request);

    // Assert
    assert_matches!(error, audited_error => {
        assert_matches!(audited_error.event, AuditEvent::Final(
            ChainedAuditEvent {
                external_token: Some(TokenAuditEvent {
                    reason_errors,
                    ..
                }),
                ..
            }) if reason_errors.contains("token-not-present"));
        assert_eq!(audited_error.cause.to_string(), "Token not present");
    });
}
