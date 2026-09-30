use super::enrich_from_origin;
use crate::http::middleware::audit::audited_error::AuditedError;
use crate::http::middleware::audit::audited_response::AuditedResponse;
use crate::http::middleware::extract_external_token::external_token_error::ExternalTokenError;
use crate::services::audit::chained::audit_event::AuditEvent;
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::audit::chained::audit_event::intermediate_audit_event::IntermediateAuditEvent;
use actix_web::dev::ServiceResponse;
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::middleware::from_fn;
use actix_web::{App, Error, HttpMessage, HttpResponse, test};

#[actix_web::test]
async fn test_token_extraction_failure_request_context() {
    for internal in [false, true] {
        let app = test::init_service(
            App::new()
                .wrap_fn(move |request, _srv| {
                    request
                        .extensions_mut()
                        .insert(AuditEvent::Intermediate(IntermediateAuditEvent::empty()));
                    std::future::ready(Err::<ServiceResponse, Error>(
                        AuditedError::token_not_present(&request, internal).into(),
                    ))
                })
                .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>)),
        )
        .await;
        let request = test::TestRequest::get()
            .insert_header(("X-Original-URL", "https://example.com/resource"))
            .insert_header(("User-Agent", "test-agent"))
            .to_request();
        let error = test::try_call_service(&app, request).await.unwrap_err();
        let error = error.as_error::<AuditedError>().unwrap();
        let AuditEvent::Final(event) = &error.event else {
            panic!("Expected a final audit event");
        };
        assert_eq!(event.original_url.as_deref(), Some("https://example.com/resource"));
        assert_eq!(event.user_agent.as_deref(), Some("test-agent"));
    }
}

#[actix_web::test]
async fn test_response_context_and_body() {
    for is_final in [false, true] {
        let app = test::init_service(
            App::new()
                .wrap_fn(move |request, _srv| {
                    let event = if is_final {
                        AuditEvent::Final(FinalAuditEvent::for_test())
                    } else {
                        AuditEvent::Intermediate(IntermediateAuditEvent::empty())
                    };
                    request.extensions_mut().insert(event);
                    std::future::ready(Ok::<_, Error>(
                        request.into_response(HttpResponse::Accepted().body("response body")),
                    ))
                })
                .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>)),
        )
        .await;
        let request = test::TestRequest::get()
            .insert_header(("X-Original-URL", "https://example.com/resource"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let event = response.request().extensions().get::<AuditEvent>().unwrap().clone();
        match event {
            AuditEvent::Final(event) => {
                assert!(is_final);
                assert_eq!(event.original_url.as_deref(), Some("https://example.com/resource"));
                assert_eq!(event.user_agent, None);
            }
            AuditEvent::Intermediate(event) => {
                assert!(!is_final);
                assert!(event.is_empty());
            }
        }
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(test::read_body(response).await, "response body");
    }
}

#[actix_web::test]
async fn test_enriched_error_preserves_response() {
    let app = test::init_service(
        App::new()
            .wrap_fn(|_request, _srv| {
                let error = AuditedError::wrap(TestResponseError);
                std::future::ready(Err::<ServiceResponse, Error>(error.into()))
            })
            .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>)),
    )
    .await;
    let request = test::TestRequest::get().to_request();
    let error = test::try_call_service(&app, request).await.unwrap_err();
    assert!(error.as_error::<AuditedError>().is_some());
    let response = error.error_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers().get("X-Error-Details").unwrap(), "preserved");
    assert_eq!(
        actix_web::body::to_bytes(response.into_body()).await.unwrap(),
        "original error body"
    );
}

#[derive(Debug)]
struct TestResponseError;

impl std::fmt::Display for TestResponseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("original cause")
    }
}

impl actix_web::ResponseError for TestResponseError {
    fn error_response(&self) -> HttpResponse {
        let mut response = HttpResponse::Forbidden()
            .insert_header(("X-Error-Details", "preserved"))
            .body("original error body");
        response
            .extensions_mut()
            .insert(AuditEvent::Final(FinalAuditEvent::for_test()));
        response
    }
}

#[actix_web::test]
async fn test_unrelated_error_is_propagated() {
    let app = test::init_service(
        App::new()
            .wrap_fn(|_request, _srv| {
                std::future::ready(Err::<ServiceResponse, Error>(
                    InternalError::new("original cause", StatusCode::BAD_REQUEST).into(),
                ))
            })
            .wrap(from_fn(enrich_from_origin::<AuditedResponse<_>, AuditedError, _>)),
    )
    .await;
    let error = test::try_call_service(&app, test::TestRequest::get().to_request())
        .await
        .unwrap_err();
    assert!(error.as_error::<InternalError<&str>>().is_some());
    assert_eq!(error.to_string(), "original cause");
    assert_eq!(error.error_response().status(), StatusCode::BAD_REQUEST);
}
