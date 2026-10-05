use super::StatusFilter;
use actix_web::http::StatusCode;
use std::marker::PhantomData;

#[cfg(test)]
mod tests;

/// Provides the statuses that may pass without audit finalization.
pub trait AllowedStatuses {
    const STATUSES: &'static [StatusCode];
}

/// Finalizes statuses outside the allowed list; listed statuses pass unchanged.
///
/// An empty list finalizes every response status.
///
/// ```
/// use actix_web::http::StatusCode;
/// use boxer_core::http::middleware::audit::finalize_on_status;
/// use boxer_core::http::middleware::audit::status_filter::skip_unmatched_statuses::{
///     AllowedStatuses, SkipUnmatched,
/// };
///
/// struct Allowed;
///
/// impl AllowedStatuses for Allowed {
///     const STATUSES: &'static [StatusCode] = &[StatusCode::UNAUTHORIZED, StatusCode::OK];
/// }
///
/// let app = actix_web::App::new().wrap(actix_web::middleware::from_fn(
///     finalize_on_status::<SkipUnmatched<Allowed>>,
/// ));
/// ```
pub struct SkipUnmatched<S: AllowedStatuses>(PhantomData<S>);

impl<S: AllowedStatuses> StatusFilter for SkipUnmatched<S> {
    fn should_finalize(status: StatusCode) -> bool {
        !S::STATUSES.contains(&status)
    }
}
