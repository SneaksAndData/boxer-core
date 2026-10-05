use super::errors_as_unauthorized;
use actix_web::dev::ServiceResponse;
use actix_web::error::InternalError;
use actix_web::http::{StatusCode, header};
use actix_web::middleware::from_fn;
use actix_web::{App, HttpResponse, test, web};

#[actix_web::test]
async fn changes_error_status_and_removes_body() {
    for status in [
        StatusCode::BAD_REQUEST,
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::NOT_FOUND,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::SERVICE_UNAVAILABLE,
    ] {
        let app = test::init_service(
            App::new().service(
                web::scope("/protected")
                    .wrap_fn(move |_req, _srv| {
                        let response = HttpResponse::build(status)
                            .insert_header(("x-error", "preserved"))
                            .insert_header((header::CONTENT_LENGTH, "13"))
                            .insert_header((header::CONTENT_ENCODING, "identity"))
                            .body("Error details");
                        std::future::ready(Err::<ServiceResponse, actix_web::Error>(
                            InternalError::from_response("cause", response).into(),
                        ))
                    })
                    .wrap(from_fn(errors_as_unauthorized)),
            ),
        )
        .await;
        let error = test::try_call_service(&app, test::TestRequest::with_uri("/protected").to_request())
            .await
            .expect_err("The result must remain an error");
        assert_eq!(error.as_response_error().status_code(), StatusCode::UNAUTHORIZED);
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers().get("x-error").unwrap(), "preserved");
        assert!(!response.headers().contains_key(header::CONTENT_LENGTH));
        assert!(!response.headers().contains_key(header::TRANSFER_ENCODING));
        assert!(!response.headers().contains_key(header::CONTENT_ENCODING));
        assert!(
            actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[actix_web::test]
async fn leaves_all_ok_responses_unchanged() {
    for status in [
        StatusCode::OK,
        StatusCode::FOUND,
        StatusCode::FORBIDDEN,
        StatusCode::INTERNAL_SERVER_ERROR,
    ] {
        let app = test::init_service(
            App::new().service(
                web::scope("/protected")
                    .route(
                        "",
                        web::to(move || async move { HttpResponse::build(status).body("Response") }),
                    )
                    .wrap(from_fn(errors_as_unauthorized)),
            ),
        )
        .await;
        let response = test::call_service(&app, test::TestRequest::with_uri("/protected").to_request()).await;
        assert_eq!(response.status(), status);
        assert_eq!(test::read_body(response).await, "Response");
    }
}

#[actix_web::test]
async fn does_not_change_errors_in_sibling_scopes() {
    let app = test::init_service(
        App::new()
            .service(web::scope("/protected").wrap(from_fn(errors_as_unauthorized)))
            .service(web::scope("/other").wrap_fn(|_req, _srv| {
                std::future::ready(Err::<ServiceResponse, _>(actix_web::error::ErrorForbidden("Denied")))
            })),
    )
    .await;
    let error = test::try_call_service(&app, test::TestRequest::with_uri("/other").to_request())
        .await
        .unwrap_err();
    assert_eq!(error.as_response_error().status_code(), StatusCode::FORBIDDEN);
}
