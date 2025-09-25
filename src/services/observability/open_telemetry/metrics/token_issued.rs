use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};

#[derive(Clone)]
pub struct TokenIssued(Counter<u64>, String);

impl TokenIssued {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenIssued {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_issued"))
            .with_description("Count of successfully issued tokens")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenIssuedMetric {
    fn increment(&self, identity_provider: String, external_identity: String);
}

impl TokenIssuedMetric for TokenIssued {
    fn increment(&self, identity_provider: String, external_identity: String) {
        self.0.add(
            1,
            &[
                KeyValue::new("external_identity", external_identity),
                KeyValue::new("identity_provider", identity_provider),
                KeyValue::new("instance_id", self.1.clone()),
            ],
        );
    }
}
