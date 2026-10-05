use crate::http::middleware::audit::status_filter::StatusFilter;
use actix_web::http::StatusCode;

/// Skips all successful (2xx) responses.
pub(super) struct SkipSuccessfulStatuses;

impl StatusFilter for SkipSuccessfulStatuses {
    fn should_finalize(status: StatusCode) -> bool {
        !status.is_success()
    }
}
