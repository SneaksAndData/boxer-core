use crate::http::middleware::audit::audit_recorder::audit_event_source::AuditEventSource;
use crate::http::middleware::audit::enrich_from_origin::OriginContext;
use crate::http::middleware::audit::enrich_from_origin::enrich_from_origin::EnrichFromOrigin;
use crate::services::audit::chained::audit_event::AuditEvent;
use actix_web::HttpMessage;
use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;

/// [`AuditedResponse`] contains an abstraction layer for `ServiceResponse` that abstracts the handling
/// of the audit metadata and the audit metadata validation.
pub struct AuditedResponse<BodyType = BoxBody>(ServiceResponse<BodyType>);

impl<BodyType> EnrichFromOrigin<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    fn enrich_from_origin(response: ServiceResponse<BodyType>, context: OriginContext) -> ServiceResponse<BodyType> {
        if let Some(AuditEvent::Final(event)) = response.request().extensions_mut().get_mut::<AuditEvent>() {
            context.apply(event);
        }
        response
    }
}

impl<BodyType> AuditEventSource<AuditEvent> for AuditedResponse<BodyType> {
    fn audit_event(&self) -> AuditEvent {
        self.0
            .request()
            .extensions()
            .get::<AuditEvent>()
            .cloned()
            .expect("Audited event not exists in request extensions")
    }
}

impl<BodyType> From<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    /// Attempts to create an [`AuditedResponse`] from a [`ServiceResponse`] by checking if the request
    /// contains an [`FinalAuditEvent`] in its extensions. If the [`FinalAuditEvent`] is not found,
    /// an error is returned indicating that the response cannot be audited.
    fn from(value: ServiceResponse<BodyType>) -> Self {
        let contains = { value.request().extensions().contains::<AuditEvent>() };

        match contains {
            false => panic!("AuditedResponse: Audit event not found in ServiceRequest request extensions"),
            true => Self(value),
        }
    }
}

impl<BodyType> Into<ServiceResponse<BodyType>> for AuditedResponse<BodyType> {
    /// Converts the [`AuditedResponse`] to a [`ServiceResponse`] by extracting its underlying object
    fn into(self) -> ServiceResponse<BodyType> {
        self.0
    }
}
