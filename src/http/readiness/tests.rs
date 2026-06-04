use actix_web::{App, test, web};

use crate::http::readiness::{ReadinessProbe, readiness_handler};

#[actix_web::test]
async fn test_readiness_lifecycle() {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let probe = ReadinessProbe::new(rx);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(probe.clone()))
            .route("/readiness", web::get().to(readiness_handler)),
    )
    .await;

    // Before signal: not ready
    let req = test::TestRequest::get().uri("/readiness").to_request();
    let resp: bool = test::call_and_read_body_json(&app, req).await;
    assert!(!resp, "should not be ready before signal");

    // Backend signals readiness
    tx.send(()).unwrap();

    // After signal: ready
    let req = test::TestRequest::get().uri("/readiness").to_request();
    let resp: bool = test::call_and_read_body_json(&app, req).await;
    assert!(resp, "should be ready after signal");

    // Cached: still ready via AtomicBool fast-path
    let req = test::TestRequest::get().uri("/readiness").to_request();
    let resp: bool = test::call_and_read_body_json(&app, req).await;
    assert!(resp, "should remain ready (cached)");
}
