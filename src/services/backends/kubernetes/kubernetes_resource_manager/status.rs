use kube::core::ErrorResponse;
use kube::runtime::reflector::ObjectRef;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The backend error type for Kubernetes resource management operations.
/// These errors should be mapped to the REST API status codes
#[derive(Debug)]
pub enum Status {
    Conflict,
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

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Conflict => write!(f, "Conflict error occurred"),
            Status::Other(e) => write!(f, "An error occurred: {}", e),
            Status::NotFound(details) => write!(f, "Resource not found: {}", details),
            Status::Deleted(details) => write!(f, "Resource was deleted not found: {}", details),
            Status::ConversionError(cause) => write!(f, "Conversion error occurred: {}", cause),
            Status::Timeout(message) => write!(f, "Operation timed out: {}", message),
        }
    }
}

impl Error for Status {}

#[derive(Debug)]
pub struct NotFoundDetails {
    pub name: String,
    pub namespace: Option<String>,
}

impl NotFoundDetails {
    pub fn new(name: String, namespace: Option<String>) -> Self {
        NotFoundDetails { name, namespace }
    }
}

impl Display for NotFoundDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let namespace = self.namespace.as_deref().unwrap_or("unknown");
        write!(
            f,
            "Resource name: '{}',  namespace '{}' not found",
            self.name, namespace
        )
    }
}

impl<R> From<&ObjectRef<R>> for NotFoundDetails
where
    R: kube::Resource,
{
    fn from(object_ref: &ObjectRef<R>) -> Self {
        NotFoundDetails {
            name: object_ref.name.clone(),
            namespace: object_ref.namespace.clone(),
        }
    }
}
