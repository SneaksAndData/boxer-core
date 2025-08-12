use crate::services::backends::kubernetes::kubernetes_resource_manager::ListenerConfig;
use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub operation_timeout: DurationString,
}
impl Into<ListenerConfig> for &RepositorySettings {
    fn into(self) -> ListenerConfig {
        ListenerConfig {
            label_selector_key: self.label_selector_key.clone(),
            label_selector_value: self.label_selector_value.clone(),
            operation_timeout: self.operation_timeout.into(),
        }
    }
}
