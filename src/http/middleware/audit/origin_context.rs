#[cfg(test)]
mod tests;

use crate::services::audit::chained::audit_event::final_audit_event::FinalAuditEvent;
use crate::services::validation_service::request_context::extract_original_url;
use actix_web::dev::ServiceRequest;
use actix_web::http::header::USER_AGENT;

pub struct OriginContext {
    original_url: Option<String>,
    user_agent: Option<String>,
}

impl OriginContext {
    pub fn from_request(request: &ServiceRequest) -> Self {
        let original_url = match extract_original_url(request.request()) {
            Ok(url) => Some(preview(&url)),
            Err(error) => {
                log::debug!("Original URL unavailable for audit context: {error}");
                None
            }
        };
        let user_agent = request
            .headers()
            .get(USER_AGENT)
            .and_then(|value| match value.to_str() {
                Ok(value) => Some(preview(value)),
                Err(error) => {
                    log::warn!("Invalid User-Agent header in audit context: {error}");
                    None
                }
            });

        Self {
            original_url,
            user_agent,
        }
    }

    pub fn apply(self, event: &mut FinalAuditEvent) {
        event.original_url = self.original_url;
        event.user_agent = self.user_agent;
    }
}

fn preview(value: &str) -> String {
    value.chars().take(100).collect()
}
