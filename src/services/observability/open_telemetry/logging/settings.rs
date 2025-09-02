use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogSettings {
    pub enabled: bool,
}
