use crate::services::observability::open_telemetry::metrics::token_accepted::TokenAccepted;
use crate::services::observability::open_telemetry::metrics::token_attempt::TokenAttempt;
use crate::services::observability::open_telemetry::metrics::token_forbidden::TokenForbidden;
use crate::services::observability::open_telemetry::metrics::token_issued::TokenIssued;
use crate::services::observability::open_telemetry::metrics::token_lifetime::TokenLifetime;
use crate::services::observability::open_telemetry::metrics::token_rejected::TokenRejected;
use crate::services::observability::open_telemetry::metrics::token_unauthorized::TokenUnauthorized;
use crate::services::service_provider::ServiceProvider;

pub struct MetricsProvider {
    token_forbidden: TokenForbidden,
    token_unauthorized: TokenUnauthorized,
    token_issued: TokenIssued,
    token_attempt: TokenAttempt,
    token_lifetime: TokenLifetime,
    token_accepted: TokenAccepted,
    token_rejected: TokenRejected,
}

impl MetricsProvider {
    // COVERAGE: Disable since the function is trivial
    #[cfg_attr(coverage, coverage(off))]
    pub fn new(root_metrics_namespace: &'static str, instance_id: String) -> Self {
        Self {
            token_forbidden: TokenForbidden::new(root_metrics_namespace, instance_id.clone()),
            token_unauthorized: TokenUnauthorized::new(root_metrics_namespace, instance_id.clone()),
            token_issued: TokenIssued::new(root_metrics_namespace, instance_id.clone()),
            token_attempt: TokenAttempt::new(root_metrics_namespace, instance_id.clone()),
            token_lifetime: TokenLifetime::new(root_metrics_namespace, instance_id.clone()),
            token_accepted: TokenAccepted::new(root_metrics_namespace, instance_id.clone()),
            token_rejected: TokenRejected::new(root_metrics_namespace, instance_id),
        }
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenForbidden> for MetricsProvider {
    fn get(&self) -> TokenForbidden {
        self.token_forbidden.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenUnauthorized> for MetricsProvider {
    fn get(&self) -> TokenUnauthorized {
        self.token_unauthorized.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenIssued> for MetricsProvider {
    fn get(&self) -> TokenIssued {
        self.token_issued.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenAttempt> for MetricsProvider {
    fn get(&self) -> TokenAttempt {
        self.token_attempt.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenLifetime> for MetricsProvider {
    fn get(&self) -> TokenLifetime {
        self.token_lifetime.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenAccepted> for MetricsProvider {
    fn get(&self) -> TokenAccepted {
        self.token_accepted.clone()
    }
}

// COVERAGE: Disable since the function is trivial
#[cfg_attr(coverage, coverage(off))]
impl ServiceProvider<TokenRejected> for MetricsProvider {
    fn get(&self) -> TokenRejected {
        self.token_rejected.clone()
    }
}
