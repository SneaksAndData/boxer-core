use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audit_recorder::audit_recorder_factory::AuditRecorderFactory;
use crate::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;
use actix_web::error::ErrorInternalServerError;
use actix_web::{App, Error, HttpMessage, HttpResponse, test, web};
use anyhow::Result;
use mockall::mock;
use pretty_assertions::assert_matches;
use std::sync::Arc;

#[actix_web::test]
async fn test_audit_success() {
    // Arrange
    let mut audit = MockAuditWriter::new();
    audit.expect_write().returning(|_| ());

    let chain = App::new()
        .wrap(AuditRecorderFactory::<MockAuditEventSource>::new(Arc::new(audit)))
        .default_service(web::to(|| async move { HttpResponse::Ok().finish() }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act
    let _ = test::call_service(&service, request).await;

    // Expectations are verified automatically when `mock_audit` is dropped at the end of the scope.
}

#[actix_web::test]
async fn test_audit_error() {
    // Arrange
    let mut audit = MockAuditWriter::new();
    audit.expect_write().returning(|_| ());

    let chain = App::new()
        .wrap(AuditRecorderFactory::<MockAuditEventSource>::new(Arc::new(audit)))
        .default_service(web::to(|| async move {
            Result::<HttpResponse, Error>::Err(ErrorInternalServerError("Some error"))
        }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act

    let _ = test::call_service(&service, request).await;

    // Expectations are verified automatically when `mock_audit` is dropped at the end of the scope.
}

#[actix_web::test]
#[should_panic(expected = "Error without audit should not reach audit recorder middleware: \"Some error\"")]
async fn test_custom_error_without_error_event() {
    // Arrange
    let mut audit = MockAuditWriter::new();
    audit.expect_write().returning(|_| ());

    let chain = App::new()
        .wrap_fn(|_req, _srv| {
            std::future::ready(Err::<ServiceResponse<BoxBody>, _>(ErrorInternalServerError(
                "Some error",
            )))
        })
        .wrap(AuditRecorderFactory::<MockAuditEventSource>::new(Arc::new(audit)))
        .default_service(web::to(|| async move { HttpResponse::Ok().finish() }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act

    let _: ServiceResponse<BoxBody> = test::call_service(&service, request).await;

    // The code above should panic.
}

#[actix_web::test]
#[should_panic(expected = "Attempt to wrap an error without an audit event")]
async fn test_custom_error_panic_request_without_event() {
    // Arrange
    let mut audit = MockAuditWriter::new();
    audit.expect_write().returning(|_| ());

    let chain = App::new()
        .wrap_fn(|req, _src| {
            let error = AuditedError::from_request(&req, ErrorInternalServerError("Some error"));
            std::future::ready(Err::<ServiceResponse<BoxBody>, _>(Error::from(error)))
        })
        .wrap(AuditRecorderFactory::<MockAuditEventSource>::new(Arc::new(audit)))
        .default_service(web::to(|| async move { HttpResponse::Ok().finish() }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act
    let _: ServiceResponse<BoxBody> = test::call_service(&service, request).await;

    // The code above should panic.
}

#[actix_web::test]
async fn test_custom_error_recording() {
    // Arrange
    let mut audit = MockAuditWriter::new();
    audit.expect_write().returning(|_| ());

    let chain = App::new()
        .wrap_fn(|req, _src| {
            req.extensions_mut()
                .insert(AuditEvent::Intermediate(ChainedAuditEvent::empty()));
            let error = AuditedError::from_request(&req, ErrorInternalServerError("Some error"));
            std::future::ready(Err::<ServiceResponse<BoxBody>, _>(Error::from(error)))
        })
        .wrap(AuditRecorderFactory::<MockAuditEventSource>::new(Arc::new(audit)))
        .default_service(web::to(|| async move { HttpResponse::Ok().finish() }));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act
    let result = test::try_call_service(&service, request).await;

    // Assert
    let err = result.expect_err("Expected service call to fail with an error");
    let audited_err = err
        .as_error::<AuditedError>()
        .expect("Expected error to be downcastable to AuditedError");
    assert_matches!(
        audited_err,
        AuditedError {
            event: AuditEvent::Intermediate(_),
            ..
        }
    );
}

mock! {
    pub AuditEventSource {}

    impl AuditEventSource for AuditEventSource {
        fn audit_event(&self) -> AuditEvent;
    }
}

mock! {

    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }
}

impl<B> TryFrom<ServiceResponse<B>> for MockAuditEventSource {
    type Error = actix_web::Error;

    fn try_from(_value: ServiceResponse<B>) -> Result<Self, Self::Error> {
        let mut mock = MockAuditEventSource::new();
        mock.expect_audit_event()
            .returning(|| AuditEvent::Intermediate(ChainedAuditEvent::empty()));
        Ok(mock)
    }
}

impl Into<ServiceResponse<BoxBody>> for MockAuditEventSource {
    fn into(self) -> ServiceResponse<BoxBody> {
        ServiceResponse::new(
            test::TestRequest::get().uri("/token").to_http_request(),
            HttpResponse::Ok().finish(),
        )
    }
}
