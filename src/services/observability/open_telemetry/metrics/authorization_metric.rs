use crate::services::observability::open_telemetry::metrics::into_metric_tag::IntoMetricTag;

pub trait AuthorizationMetric {
    fn increment<T, E, F>(&self, principal: T, action: E, resource: F)
    where
        T: IntoMetricTag,
        E: IntoMetricTag,
        F: IntoMetricTag;
}
