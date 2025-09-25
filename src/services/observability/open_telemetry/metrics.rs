pub mod settings;

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

pub struct TokenIssuanceCounter(Counter<u64>);

impl TokenIssuanceCounter {
    pub fn new(app_name: &'static str) -> TokenIssuanceCounter {
        let meter = global::meter(app_name);
        Self(
            meter
                .u64_counter(format!("{}.{}", app_name, "token_succeeded"))
                .with_description("Count of successfully processed tokens")
                .with_unit("tokens")
                .build(),
        )
    }
}

pub trait TokenIssuanceMetric {
    fn increment(&self, external_identity: String, identity_provider: String);
}

impl TokenIssuanceMetric for TokenIssuanceCounter {
    fn increment(&self, external_identity: String, identity_provider: String) {
        self.0.add(
            1,
            &[
                opentelemetry::KeyValue::new("external_identity", external_identity),
                opentelemetry::KeyValue::new("identity_provider", identity_provider),
            ],
        );
    }
}
