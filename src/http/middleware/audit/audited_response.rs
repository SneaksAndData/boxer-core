use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::services::audit::chained::audit_event::AuditEvent;
use actix_web::HttpMessage;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;

/// [`AuditedResponse`] contains an abstraction layer for `ServiceResponse` that abstracts the handling
/// of the audit metadata and the audit metadata validation.
pub struct AuditedResponse<BodyType = BoxBody>(ServiceResponse<BodyType>);

impl<BodyType> AuditEventSource<AuditEvent> for AuditedResponse<BodyType> {
    type Error = actix_web::Error;

    fn audit_event(&self) -> Result<AuditEvent, Self::Error> {
        self.0
            .request()
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .ok_or_else(|| actix_web::error::ErrorInternalServerError("Audited event not exists in request extensions"))
    }
}

impl<BodyType> TryFrom<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    type Error = actix_web::Error;

    /// Attempts to create an [`AuditedResponse`] from a [`ServiceResponse`] by checking if the request
    /// contains an [`FinalAuditEvent`] in its extensions. If the [`FinalAuditEvent`] is not found,
    /// an error is returned indicating that the response cannot be audited.
    fn try_from(value: ServiceResponse<BodyType>) -> Result<Self, Self::Error> {
        let contains = { value.request().extensions().contains::<AuditEvent>() };

        match contains {
            false => Err(actix_web::error::ErrorInternalServerError(
                "AuditedResponse: Audit event not found in ServiceRequest request extensions",
            )),
            true => Ok(Self(value)),
        }
    }
}

impl<BodyType> Into<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    /// Converts the [`AuditedResponse`] to a [`ServiceResponse`] by extracting its underlying object
    fn into(self) -> ServiceResponse<BodyType> {
        self.0
    }
}
