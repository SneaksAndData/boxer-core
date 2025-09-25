use opentelemetry::metrics::Counter;
use opentelemetry::{KeyValue, global};

#[derive(Clone)]
pub struct TokenAttempt(Counter<u64>, String);

impl TokenAttempt {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenAttempt {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_attempt"))
            .with_description("Attempt to issue a token, successful or not")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenAttemptMetric {
    fn increment(&self, identity_provider: String);
}

impl TokenAttemptMetric for TokenAttempt {
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
