use super::*;

struct Allowed;

impl AllowedStatuses for Allowed {
    const STATUSES: &'static [StatusCode] = &[StatusCode::UNAUTHORIZED, StatusCode::OK];
}

struct Empty;

impl AllowedStatuses for Empty {
    const STATUSES: &'static [StatusCode] = &[];
}

#[test]
fn finalizes_only_statuses_outside_the_list() {
    for code in 100..=999 {
        let status = StatusCode::from_u16(code).unwrap();
        assert_eq!(
            SkipUnmatched::<Allowed>::should_finalize(status),
            code != 401 && code != 200,
            "Unexpected decision for {status}"
        );
    }
}

#[test]
fn empty_list_finalizes_every_status() {
    for code in 100..=999 {
        let status = StatusCode::from_u16(code).unwrap();
        assert!(SkipUnmatched::<Empty>::should_finalize(status));
    }
}
