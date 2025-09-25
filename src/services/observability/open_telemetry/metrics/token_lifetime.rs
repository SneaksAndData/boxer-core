use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};
use std::time::Duration;

#[derive(Clone)]
pub struct TokenLifetime(Counter<u64>, String);

impl TokenLifetime {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenLifetime {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_lifetime"))
            .with_description("The lifetime of issued tokens in seconds")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenLifetimeMetric {
    fn increment(&self, identity_provider: String, external_identity: String, duration: Duration);
}

impl TokenLifetimeMetric for TokenLifetime {
    fn increment(&self, identity_provider: String, external_identity: String, duration: Duration) {
        self.0.add(
            duration.as_secs(),
            &[
                KeyValue::new("external_identity", external_identity),
                KeyValue::new("identity_provider", identity_provider),
                KeyValue::new("instance_id", self.1.clone()),
            ],
        );
    }
}
