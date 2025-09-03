use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TracingSettings {
    pub enabled: bool,
}
