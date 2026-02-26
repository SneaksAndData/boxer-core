pub mod not_found_details;
pub mod owner_conflict_details;

use crate::services::backends::kubernetes::kubernetes_resource_manager::status::not_found_details::NotFoundDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::owner_conflict_details::OwnerConflictDetails;
use kube::core::ErrorResponse;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The backend error type for Kubernetes resource management operations.
/// These errors should be mapped to the REST API status codes
#[derive(Debug)]
pub enum Status {
    Conflict,
    NotOwned(OwnerConflictDetails),
    Other(kube::Error),
    NotFound(NotFoundDetails),
    Deleted(NotFoundDetails),
    ConversionError(anyhow::Error),
    Timeout(String),
}

impl Status {
    pub fn is_not_found(&self) -> bool {
        match self {
            Status::NotFound(_) => true,
            _ => false,
        }
    }
}

impl From<kube::Error> for Status {
    fn from(error: kube::Error) -> Self {
        match error {
            kube::Error::Api(ErrorResponse { code: 409, .. }) => Status::Conflict,
            _ => Status::Other(error),
        }
    }
}

impl From<anyhow::Error> for Status {
    fn from(error: anyhow::Error) -> Self {
        Status::ConversionError(error)
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Conflict => write!(f, "Conflict error occurred"),
            Status::NotOwned(details) => write!(f, "Owner conflict: {}", details),
            Status::Other(e) => write!(f, "An error occurred: {}", e),
            Status::NotFound(details) => write!(f, "Resource not found: {}", details),
            Status::Deleted(details) => write!(f, "Resource was deleted not found: {}", details),
            Status::ConversionError(cause) => write!(f, "Conversion error occurred: {}", cause),
            Status::Timeout(message) => write!(f, "Operation timed out: {}", message),
        }
    }
}

impl Error for Status {}
