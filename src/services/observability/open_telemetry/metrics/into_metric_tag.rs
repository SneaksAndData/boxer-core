use cedar_policy::EntityUid;

pub trait IntoMetricTag {
    fn into_metric_tag(self) -> String;
}

impl IntoMetricTag for EntityUid {
    fn into_metric_tag(self) -> String {
        format!("{}.{}", self.type_name(), self.id().unescaped())
    }
}
