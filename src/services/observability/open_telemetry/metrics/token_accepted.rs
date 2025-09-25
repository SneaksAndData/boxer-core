use crate::services::observability::open_telemetry::metrics::authorization_metric::AuthorizationMetric;
use crate::services::observability::open_telemetry::metrics::into_metric_tag::IntoMetricTag;
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

impl AuthorizationMetric for TokenAccepted {
    fn increment<T, E, F>(&self, principal: T, action: E, resource: F)
    where
        T: IntoMetricTag,
        E: IntoMetricTag,
        F: IntoMetricTag,
    {
        self.0.add(
            1,
            &[
                KeyValue::new("principal", principal.into_metric_tag()),
                KeyValue::new("action", action.into_metric_tag()),
                KeyValue::new("resource", resource.into_metric_tag()),
                KeyValue::new("instance_id", self.1.clone()),
            ],
        );
    }
}
