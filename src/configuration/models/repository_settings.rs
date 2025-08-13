use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepositorySettings {
    pub operation_timeout: DurationString,
}
