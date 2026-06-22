#[cfg(test)]
mod tests;

use crate::http::middleware::audit::external_token::external_token_error::ExternalTokenError;
use crate::services::audit::chained::audit_event::AuditEvent;
use actix_web::dev::ServiceRequest;
use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::{HttpMessage, ResponseError};
use anyhow::anyhow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// [`AuditedError`] is a wrapper for any error that implements `ResponseError` and carries
/// an associated [`AuditEvent`] that can be recorded by the audit middleware.
#[derive(Debug)]
pub struct AuditedError {
    pub event: AuditEvent,
    cause: Box<dyn ResponseError>,
}

impl AuditedError {
    /// Wraps a given `ResponseError` into an `AuditedError`, extracting the associated
    /// `AuditEvent` from the error's response extensions.
    pub fn wrap(cause: impl ResponseError + 'static) -> AuditedError {
        let event = cause
            .error_response()
            .extensions()
            .get::<AuditEvent>()
            .expect("Attempt to wrap an error without an audit event")
            .clone();
        AuditedError {
            event,
            cause: Box::new(cause),
        }
    }

    /// Similar to `wrap` but extracts the `AuditEvent` from the request's extensions instead of
    /// the error's response.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an AuditEvent extension.
    /// Panics if the contained AuditEvent is AuditEvent::Final, since final
    /// audit events are not intended to be wrapped as errors.
    pub fn from_request(request: &ServiceRequest, cause: impl Error + 'static) -> AuditedError {
        let event = request
            .extensions()
            .get::<AuditEvent>()
            .expect("Attempt to wrap an error without an audit event")
            .clone();
        match event {
            AuditEvent::Final(_) => {
                panic!("Final audit event in a request should not be wrapped for an error")
            }
            AuditEvent::Intermediate(data) => AuditedError {
                event: AuditEvent::Intermediate(data),
                cause: Box::new(InternalError::new(cause, StatusCode::INTERNAL_SERVER_ERROR)),
            },
        }
    }
}

impl ExternalTokenError for AuditedError {
    /// Creates an `AuditedError` in case when the external token is not present.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an AuditEvent extension.
    /// Panics if the contained AuditEvent is AuditEvent::Final, since final
    /// audit events are not intended to be wrapped as errors.
    /// Panics if token is not present, but audit event is not empty
    fn external_token_not_present(request: &ServiceRequest) -> AuditedError {
        let event = request
            .extensions()
            .get::<AuditEvent>()
            .expect("Attempt to wrap a request for an error without audit event")
            .clone();
        match event {
            AuditEvent::Final(_) => {
                panic!("Final audit event in a request should not be wrapped for token not present error")
            }
            AuditEvent::Intermediate(data) if data.is_empty() => AuditedError {
                event: AuditEvent::token_not_present(),
                cause: Box::new(InternalError::new(
                    anyhow!("Token not present"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )),
            },
            AuditEvent::Intermediate(data) => {
                panic!("Non-empty audit event when token is not present: {:?}", data)
            }
        }
    }

    /// Creates an `AuditedError` for requests where an external token is present
    /// but cannot be parsed.
    ///
    /// The method reads the current request `AuditEvent` from extensions and
    /// converts an empty intermediate event into a final token-extraction-failed event,
    /// preserving the original parsing error as the underlying HTTP error cause.
    ///
    /// # Panics
    ///
    /// Panics if the request does not contain an `AuditEvent` extension.
    /// Panics if the contained event is `AuditEvent::Final`, because final events
    /// are not expected at this stage.
    /// Panics if the contained intermediate event is not empty because it is not expected in this
    /// context.
    fn token_extraction_failed(request: &ServiceRequest, cause: anyhow::Error) -> Self {
        let event = request
            .extensions()
            .get::<AuditEvent>()
            .expect("Attempt to wrap a request for an error without audit event")
            .clone();
        match event {
            AuditEvent::Final(_) => {
                panic!("Final audit event in a request should not be wrapped for token extracted error")
            }
            AuditEvent::Intermediate(data) if data.is_empty() => AuditedError {
                event: AuditEvent::token_extraction_failed(cause.to_string()),
                cause: Box::new(InternalError::new(cause, StatusCode::INTERNAL_SERVER_ERROR)),
            },
            AuditEvent::Intermediate(data) => {
                panic!("Non-empty audit event when token is not present: {:?}", data)
            }
        }
    }
}

/// The `Display` implementation for `AuditedError` simply formats the contained `AuditEvent`
/// for debugging purposes.
impl Display for AuditedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.event))
    }
}

/// The `ResponseError` implementation for `AuditedError` delegates to the underlying cause's
impl ResponseError for AuditedError {
    fn error_response(&self) -> actix_web::HttpResponse {
        self.cause.error_response()
    }
}
