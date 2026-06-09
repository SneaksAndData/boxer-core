use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::audit_recorder::audit_recorder_factory::AuditRecorderFactory;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::chained_audit_event::ChainedAuditEvent;
use crate::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::events::resource_modification_audit_event::ResourceModificationAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationEvent;
use crate::services::audit::AuditService;
use actix_web::dev::ServiceResponse;
use actix_web::error::ErrorInternalServerError;
use actix_web::{test, web, App, Error, HttpResponse};
use anyhow::Result;
use mockall::mock;
use std::sync::Arc;

#[actix_web::test]
async fn test_audit_success() {
    // Arrange
    let mut audit = MockAuditService::new();
    audit.expect_audit().returning(|_event, _final| ());

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
    let mut audit = MockAuditService::new();
    audit.expect_audit().returning(|_event, _final| ());

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

mock! {
    pub AuditEventSource {}

    impl AuditEventSource for AuditEventSource {
        fn audit_event(&self) -> AuditEvent;
    }
}

impl<B> TryFrom<&ServiceResponse<B>> for MockAuditEventSource {
    type Error = actix_web::Error;

    fn try_from(_value: &ServiceResponse<B>) -> Result<Self, Self::Error> {
        let mut mock = MockAuditEventSource::new();
        mock.expect_audit_event()
            .returning(|| AuditEvent::Intermediate(ChainedAuditEvent::empty()));
        Ok(mock)
    }
}

mock! {

    pub AuditService {}

    impl AuditService for AuditService {
        fn audit(&self, event: AuditEvent, success: bool);
        fn record_authorization(&self, event: AuthorizationAuditEvent) -> Result<()>;
        fn record_resource_deletion(&self, event: ResourceDeleteAuditEvent) -> Result<()>;
        fn record_resource_modification(&self, event: ResourceModificationAuditEvent) -> Result<()>;
        fn record_token_validation(&self, event: TokenValidationEvent) -> Result<()>;
    }
}
