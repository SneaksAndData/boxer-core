use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MetricsSettings {
    pub enabled: bool,
}
