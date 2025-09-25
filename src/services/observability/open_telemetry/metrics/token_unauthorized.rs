use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};

#[derive(Clone)]
pub struct TokenUnauthorized(Counter<u64>, String);

impl TokenUnauthorized {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenUnauthorized {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_unauthorized"))
            .with_description("Count of unauthorized token issuance attempts")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenUnauthorizedMetric {
    fn increment(&self, identity_provider: String);
}

impl TokenUnauthorizedMetric for TokenUnauthorized {
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
