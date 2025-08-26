use anyhow::Context;
use opentelemetry::global;
use opentelemetry::metrics::Counter;

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

pub fn token_succeeded(app_name: &'static str) -> Counter<u64> {
    let meter = global::meter(app_name);
    meter
        .u64_counter(format!("{}.{}", app_name, "token_succeeded"))
        .with_description("Count of successfully processed tokens")
        .with_unit("tokens")
        .build()
}
