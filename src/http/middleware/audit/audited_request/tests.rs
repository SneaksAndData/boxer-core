use crate::http::middleware::audit::audited_request::AuditedRequest;
use crate::http::middleware::audit::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use crate::services::audit::chained::audit_event::AuditEvent;
use actix_web::HttpMessage;
use actix_web::dev::ServiceRequest;
use actix_web::test::TestRequest;
use pretty_assertions::assert_matches;

#[test]
fn test_audit_event_initialization() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();

    // Act
    let result = AuditedRequest::try_create_audit_context(request);

    // Assert
    assert_matches!(result, Ok(_));
    let service_request: ServiceRequest = result.unwrap().into();
    let audit_event = service_request.extensions().get::<AuditEvent>().cloned();
    assert_eq!(audit_event.is_some(), true);
}

#[test]
fn test_audit_event_double_initialization() {
    // Arrange
    let request = TestRequest::get().uri("/any-route").to_srv_request();

    // Act
    let request: ServiceRequest = AuditedRequest::try_create_audit_context(request).unwrap().into();
    let result = AuditedRequest::try_create_audit_context(request);

    // Assert
    assert_matches!(result, Err(_));
    assert_eq!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to create audited request:"),
        true
    );
}
