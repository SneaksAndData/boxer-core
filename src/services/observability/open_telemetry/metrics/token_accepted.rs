use cedar_policy::EntityUid;
use opentelemetry::metrics::Counter;
use opentelemetry::{global, KeyValue};

#[derive(Clone)]
pub struct TokenAccepted(Counter<u64>, String);

impl TokenAccepted {
    pub(crate) fn new(app_name: &'static str, instance_id: String) -> TokenAccepted {
        let meter = global::meter(app_name);
        let counter = meter
            .u64_counter(format!("{}.{}", app_name, "token_accepted"))
            .with_description("Accepted token attempts")
            .with_unit("tokens")
            .build();
        Self(counter, instance_id)
    }
}

pub trait TokenAcceptedMetric {
    fn increment(&self, principal: EntityUid, action: EntityUid, resource: EntityUid);
}

impl TokenAcceptedMetric for TokenAccepted {
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
