use crate::services::legacy::configuration::models::AppSettings;
use config::{Config, Environment, File};

impl AppSettings {
    /// Creates a new instance of `AppSettings` by loading configuration from predefined sources
    pub fn new() -> Result<Self, anyhow::Error> {
        let s = Config::builder()
            .add_source(File::with_name("settings.toml"))
            .add_source(Environment::with_prefix("BOXER_VALIDATOR").separator("__"))
            .build()?;

        s.try_deserialize().map_err(|e| e.into())
    }
}
