pub mod authorization_metric;
mod into_metric_tag;
pub mod provider;
pub mod settings;

// COVERAGE: disabled since metric recorders should be tested in integration tests
#[cfg_attr(coverage, coverage(off))]
pub mod metric_recorders;

use anyhow::Context;
use opentelemetry::global;

// COVERAGE: disabled since this is just initialization code
#[cfg_attr(coverage, coverage(off))]
pub fn init_metrics() -> anyhow::Result<()> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
        .build()
        .with_context(|| "creating metric exporter")?;

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(opentelemetry_sdk::metrics::PeriodicReader::builder(exporter).build())
        .build();

    global::set_meter_provider(meter_provider);
    Ok(())
}
