use super::*;
use actix_web::test::TestRequest;

fn make_req(headers: &[(&str, &str)]) -> HttpRequest {
    let mut builder = TestRequest::get();
    for (name, value) in headers {
        builder = builder.insert_header((*name, *value));
    }
    builder.to_http_request()
}

// extract_header tests

#[test]
fn test_extract_header_primary() {
    let req = make_req(&[(ORIGINAL_URL_NGINX_HEADER, "http://example.com/path")]);
    let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
    assert_eq!(result.unwrap(), "http://example.com/path");
}

#[test]
fn test_extract_header_fallback() {
    let req = make_req(&[(ORIGINAL_URL_TRAEFIK_HEADER, "/traefik-path")]);
    let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
    assert_eq!(result.unwrap(), "/traefik-path");
}

#[test]
fn test_extract_header_primary_takes_precedence_over_fallback() {
    let req = make_req(&[
        (ORIGINAL_URL_NGINX_HEADER, "http://primary.com"),
        (ORIGINAL_URL_TRAEFIK_HEADER, "http://fallback.com"),
    ]);
    let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
    assert_eq!(result.unwrap(), "http://primary.com");
}

#[test]
fn test_extract_header_missing_returns_error() {
    let req = make_req(&[]);
    let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains(ORIGINAL_URL_NGINX_HEADER));
}

// extract_headers tests

#[test]
fn test_extract_headers_nginx_headers() {
    let req = make_req(&[
        (ORIGINAL_URL_NGINX_HEADER, "http://example.com/api"),
        (ORIGINAL_METHOD_NGINX_HEADER, "GET"),
    ]);
    let (url, method) = extract_headers(&req).unwrap();
    assert_eq!(url, "http://example.com/api");
    assert_eq!(method, "GET");
}

#[test]
fn test_extract_headers_missing_url_returns_error() {
    let req = make_req(&[(ORIGINAL_METHOD_NGINX_HEADER, "GET")]);
    let result = extract_headers(&req);
    assert!(result.is_err());
}

#[test]
fn test_extract_headers_missing_method_returns_error() {
    let req = make_req(&[(ORIGINAL_URL_NGINX_HEADER, "http://example.com")]);
    let result = extract_headers(&req);
    assert!(result.is_err());
}

#[test]
fn test_extract_headers_both_missing_returns_error() {
    let req = make_req(&[]);
    let result = extract_headers(&req);
    assert!(result.is_err());
}
