use cedar_policy::EntityUid;
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};

#[derive(Clone)]
pub struct TokenRejected(Counter<u64>, String);

impl TokenRejected {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenRejected {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_rejected"))
            .with_description("Rejected token attempts")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenRejectedMetric {
    fn increment(&self, principal: EntityUid, action: EntityUid, resource: EntityUid);
}

impl TokenRejectedMetric for TokenRejected {
    fn increment(&self, principal: EntityUid, action: EntityUid, resource: EntityUid) {
        self.0.add(
            1,
            &[
                KeyValue::new("principal", principal.to_string()),
                KeyValue::new("action", action.to_string()),
                KeyValue::new("resource", resource.to_string()),
                KeyValue::new("instance_id", self.1.clone()),
            ],
        );
    }
}
