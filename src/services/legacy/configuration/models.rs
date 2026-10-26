use crate::services::observability::open_telemetry::settings::OpenTelemetrySettings;
use crate::services::token_decryption_service::encryption_keys::EncryptionKeys;
use crate::services::token_decryption_service::token_settings::TokenValidationSettings;
use anyhow::Result;
use duration_string::DurationString;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,
    pub resource_owner_label: String,
    pub operation_timeout: DurationString,
}

#[derive(Debug, Deserialize)]
pub struct BackendSettings {
    pub kubernetes: KubernetesBackendSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub deploy_environment: String,
    pub listen_address: SocketAddr,
    pub instance_name: String,
    pub backend: BackendSettings,
    pub opentelemetry: OpenTelemetrySettings,
    pub token_settings: TokenValidationSettings,
}

impl AppSettings {
    pub fn get_signatures(&self) -> Result<EncryptionKeys> {
        serde_json::from_str::<EncryptionKeys>(self.token_settings.keys.as_ref()).map_err(|e| anyhow::anyhow!(e))
    }
}
