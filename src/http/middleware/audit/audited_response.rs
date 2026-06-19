use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::services::audit::chained::audit_event::AuditEvent;
use actix_web::HttpMessage;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;

/// `AuditedResponse` contains an abstraction layer for `ServiceResponse` that abstracts the handling
/// of the audit metadata and the audit metadata validation.
pub struct AuditedResponse<BodyType = BoxBody>(ServiceResponse<BodyType>);

impl<BodyType> AuditEventSource for AuditedResponse<BodyType> {
    fn audit_event(&self) -> AuditEvent {
        self.0
            .request()
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions")
    }
}

impl<BodyType> TryFrom<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    type Error = actix_web::Error;

    fn try_from(value: ServiceResponse<BodyType>) -> Result<Self, Self::Error> {
        let contains = { value.request().extensions().contains::<AuditEvent>() };

        match contains {
            false => Err(actix_web::error::ErrorInternalServerError(
                "Audit event not found in request extensions",
            )),
            true => Ok(Self(value)),
        }
    }
}

impl<BodyType> Into<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    fn into(self) -> ServiceResponse<BodyType> {
        self.0
    }
}
