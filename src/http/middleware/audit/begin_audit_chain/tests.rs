use crate::http::middleware::audit::begin_audit_chain::begin_audit_chain;
use crate::http::middleware::audit::begin_audit_chain::try_create_audit_context::TryCreateAuditContext;
use actix_web::dev::ServiceRequest;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpRequest, HttpResponse, test, web};
use mockall::mock;

#[actix_web::test]
async fn test_begin_audit_chain() {
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
        .default_service(web::to(
            |_request: HttpRequest| async move { HttpResponse::Ok().finish() },
        ));

    let service = test::init_service(chain).await;
    let request = test::TestRequest::get().uri("/any-route").to_request();

    // Act
    let _ = test::call_service(&service, request).await;

    // Expectations are verified automatically when `mock_audit` is dropped at the end of the scope.
}

mock! {
    pub AuditContext {}

    impl TryCreateAuditContext for AuditContext {
        fn try_create_audit_context(request: ServiceRequest) -> Result<Self, actix_web::Error>;
    }

    impl Into<ServiceRequest> for AuditContext {
        fn into(self) -> ServiceRequest;
    }
}
