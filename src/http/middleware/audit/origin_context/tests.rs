use super::{OriginContext, preview};
use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use actix_web::test::TestRequest;

#[test]
fn test_preview_character_limit() {
    for character in ['a', '\u{1f600}'] {
        for length in [0, 99, 100, 101, 200] {
            let value = character.to_string().repeat(length);
            assert_eq!(preview(&value), character.to_string().repeat(length.min(100)));
        }
    }
}

#[test]
fn test_nginx_context_and_properties() {
    let url = format!("https://example.com/{}", "a".repeat(150));
    let user_agent = "test-agent/".repeat(20);
    let request = TestRequest::get()
        .insert_header(("X-Original-URL", url.as_str()))
        .insert_header(("User-Agent", user_agent.as_str()))
        .to_srv_request();
    let mut event = FinalAuditEvent::for_test();
    OriginContext::from_request(&request).apply(&mut event);
    assert_eq!(event.original_url.as_deref(), Some(preview(&url).as_str()));
    assert_eq!(event.user_agent.as_deref(), Some(preview(&user_agent).as_str()));
    let properties = event.get_properties();
    assert_eq!(properties.original_url, Some(preview(&url)));
    assert_eq!(properties.user_agent, Some(preview(&user_agent)));
}

#[test]
fn test_traefik_context_without_user_agent() {
    let request = TestRequest::get()
        .insert_header(("X-Forwarded-Proto", "https"))
        .insert_header(("X-Forwarded-Host", "example.com"))
        .insert_header(("X-Forwarded-Uri", "/path?query=value"))
        .to_srv_request();
    let mut event = FinalAuditEvent::for_test();
    OriginContext::from_request(&request).apply(&mut event);
    assert_eq!(
        event.original_url.as_deref(),
        Some("https://example.com/path?query=value")
    );
    assert_eq!(event.user_agent, None);
}

#[test]
fn test_missing_context() {
    let request = TestRequest::get().uri("/not-the-original-url").to_srv_request();
    let mut event = FinalAuditEvent::for_test();
    OriginContext::from_request(&request).apply(&mut event);
    assert_eq!(event.original_url, None);
    assert_eq!(event.user_agent, None);
}

#[test]
fn test_invalid_headers() {
    let request = TestRequest::get()
        .insert_header(("X-Original-URL", vec![0xff]))
        .insert_header(("User-Agent", vec![0xff]))
        .to_srv_request();
    let mut event = FinalAuditEvent::for_test();
    OriginContext::from_request(&request).apply(&mut event);
    assert_eq!(event.original_url, None);
    assert_eq!(event.user_agent, None);
}
