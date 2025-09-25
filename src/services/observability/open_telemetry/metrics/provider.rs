use crate::services::observability::open_telemetry::metrics::token_accepted::TokenAccepted;
use crate::services::observability::open_telemetry::metrics::token_attempt::TokenAttempt;
use crate::services::observability::open_telemetry::metrics::token_forbidden::TokenForbidden;
use crate::services::observability::open_telemetry::metrics::token_issued::TokenIssued;
use crate::services::observability::open_telemetry::metrics::token_lifetime::TokenLifetime;
use crate::services::observability::open_telemetry::metrics::token_unauthorized::TokenUnauthorized;
use crate::services::service_provider::ServiceProvider;

pub struct MetricsProvider {
    token_forbidden: TokenForbidden,
    token_unauthorized: TokenUnauthorized,
    token_issued: TokenIssued,
    token_attempt: TokenAttempt,
    token_lifetime: TokenLifetime,
    token_accepted: TokenAccepted,
}

impl MetricsProvider {
    pub fn new(root_metrics_namespace: &'static str, instance_id: String) -> Self {
        Self {
            token_forbidden: TokenForbidden::new(root_metrics_namespace, instance_id.clone()),
            token_unauthorized: TokenUnauthorized::new(root_metrics_namespace, instance_id.clone()),
            token_issued: TokenIssued::new(root_metrics_namespace, instance_id.clone()),
            token_attempt: TokenAttempt::new(root_metrics_namespace, instance_id.clone()),
            token_lifetime: TokenLifetime::new(root_metrics_namespace, instance_id.clone()),
            token_accepted: TokenAccepted::new(root_metrics_namespace, instance_id),
        }
    }
}

impl ServiceProvider<TokenForbidden> for MetricsProvider {
    fn get(&self) -> TokenForbidden {
        self.token_forbidden.clone()
    }
}

impl ServiceProvider<TokenUnauthorized> for MetricsProvider {
    fn get(&self) -> TokenUnauthorized {
        self.token_unauthorized.clone()
    }
}

impl ServiceProvider<TokenIssued> for MetricsProvider {
    fn get(&self) -> TokenIssued {
        self.token_issued.clone()
    }
}

impl ServiceProvider<TokenAttempt> for MetricsProvider {
    fn get(&self) -> TokenAttempt {
        self.token_attempt.clone()
    }
}

impl ServiceProvider<TokenLifetime> for MetricsProvider {
    fn get(&self) -> TokenLifetime {
        self.token_lifetime.clone()
    }
}

impl ServiceProvider<TokenAccepted> for MetricsProvider {
    fn get(&self) -> TokenAccepted {
        self.token_accepted.clone()
    }
}
