use crate::services::observability::open_telemetry::logging::settings::LogSettings;
use crate::services::observability::open_telemetry::metrics::settings::MetricsSettings;
use crate::services::observability::open_telemetry::tracing::settings::TracingSettings;

pub struct OpenTelemetrySettings {
    pub log_settings: LogSettings,
    pub metrics_settings: MetricsSettings,
    pub tracing_settings: TracingSettings,
}
