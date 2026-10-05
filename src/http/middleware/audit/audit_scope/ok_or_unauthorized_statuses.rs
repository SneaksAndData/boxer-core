use crate::http::middleware::audit::status_filter::skip_unmatched_statuses::AllowedStatuses;
use actix_web::http::StatusCode;

pub(super) struct OkOrUnauthorized;

impl AllowedStatuses for OkOrUnauthorized {
    const STATUSES: &'static [StatusCode] = &[StatusCode::OK, StatusCode::UNAUTHORIZED];
}
