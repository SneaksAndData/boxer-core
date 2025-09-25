use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};

#[derive(Clone)]
pub struct TokenForbidden(Counter<u64>, String);

impl TokenForbidden {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenForbidden {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_forbidden"))
            .with_description("Count of forbidden token issuance attempts")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenForbiddenMetric {
    fn increment(&self, identity_provider: String);
}

impl TokenForbiddenMetric for TokenForbidden {
    fn increment(&self, identity_provider: String) {
        self.0.add(
            1,
            &[
                KeyValue::new("identity_provider", identity_provider),
                KeyValue::new("instance_id", self.1.clone()),
            ],
        );
    }
}
