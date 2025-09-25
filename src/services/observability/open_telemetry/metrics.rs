mod authorization_metric;
mod into_metric_tag;
pub mod provider;
pub mod settings;
pub mod token_accepted;
pub mod token_attempt;
pub mod token_forbidden;
pub mod token_issued;
pub mod token_lifetime;
pub mod token_rejected;
pub mod token_unauthorized;

use anyhow::Context;
use opentelemetry::global;

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
