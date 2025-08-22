use anyhow::Result;
use log::Log;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;

pub fn init_logger() -> Result<Box<dyn Log>> {
    let exporter = LogExporter::builder().with_tonic().build()?;
    let provider = SdkLoggerProvider::builder().with_batch_exporter(exporter).build();

    let appender = opentelemetry_appender_log::OpenTelemetryLogBridge::new(&provider);
    let logger = Box::new(appender);
    Ok(logger)
}
