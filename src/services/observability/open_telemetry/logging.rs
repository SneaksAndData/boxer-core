pub mod settings;

use anyhow::Result;
use log::Log;
use opentelemetry::KeyValue;
use opentelemetry_otlp::LogExporter;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;

#[cfg_attr(coverage, coverage(off))]
pub fn init_logger(environment: String) -> Result<Box<dyn Log>> {
    let exporter = LogExporter::builder().with_tonic().build()?;
    let environment = Resource::builder()
        .with_attribute(KeyValue::new("Environment", environment))
        .build();
    let provider = SdkLoggerProvider::builder()
        .with_resource(environment)
        .with_batch_exporter(exporter)
        .build();

    let appender = opentelemetry_appender_log::OpenTelemetryLogBridge::new(&provider);
    let logger = Box::new(appender);
    Ok(logger)
}
