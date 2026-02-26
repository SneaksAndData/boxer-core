use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;

impl From<Status> for actix_web::Error {
    // COVERAGE: Disable since the function is trivial
    #[cfg_attr(coverage, coverage(off))]
    fn from(err: Status) -> Self {
        match err {
            Status::Conflict => actix_web::error::ErrorConflict(err.to_string()),
            Status::NotOwned(_) => actix_web::error::ErrorConflict(err.to_string()),
            Status::NotFound(_) => actix_web::error::ErrorNotFound(err.to_string()),
            Status::Deleted(_) => actix_web::error::ErrorGone(err.to_string()),
            Status::Timeout(message) => actix_web::error::ErrorRequestTimeout(message),
            Status::ConversionError(cause) => actix_web::error::ErrorInternalServerError(cause),
            Status::Other(e) => actix_web::error::ErrorInternalServerError(e),
        }
    }
}
