use actix_web::http::StatusCode;

pub mod finalize_on_fail;
mod skip_sucessfull_statuses;
pub mod skip_unmatched_statuses;

/// Determines which response statuses bypass audit finalization.
pub trait StatusFilter {
    /// Returns `true` to finalize the audit event for this status.
    fn should_finalize(status: StatusCode) -> bool;
}
