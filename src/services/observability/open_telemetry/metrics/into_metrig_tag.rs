pub trait IntoMetricTag {
    fn into_metric_tag(self) -> String;
}
