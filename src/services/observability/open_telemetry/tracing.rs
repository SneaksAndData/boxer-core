pub mod settings;
pub mod tracing_facade;

use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::{Context, global};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use std::fmt::Display;

/// Initialize OpenTelemetry tracing with OTLP exporter
/// Should be called once at the start of the application
/// COVERAGE: disabled since this is just initialization code
#[cfg_attr(coverage, coverage(off))]
pub fn init_tracer() -> anyhow::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider);
    Ok(())
}

/// Start a new trace span with the given name
/// Returns a Context containing the new span
/// The caller is responsible for ending the span
/// COVERAGE: disabled since this should be tested in integration tests only
#[cfg_attr(coverage, coverage(off))]
pub fn start_trace(span_name: &str, tracer_name: Option<String>) -> Context {
    let tracer = global::tracer(tracer_name.unwrap_or("boxer".to_string()));
    let span = tracer
        .span_builder(span_name.to_string())
        .with_kind(opentelemetry::trace::SpanKind::Internal)
        .start(&tracer);
    Context::current().with_span(span)
}

/// Extension trait for Result to stop tracing and set span status
pub trait ErrorExt<T, E> {
    fn stop_trace(self, ctx: Context) -> Self;
}

/// Implementation of ErrorExt for Result
impl<T, E> ErrorExt<T, E> for Result<T, E>
where
    E: Display,
{
    #[cfg_attr(coverage, coverage(off))]
    /// COVERAGE: disabled since this should be tested in integration tests only
    fn stop_trace(self, ctx: Context) -> Self {
        if let Err(err) = &self {
            ctx.span().set_status(Status::Error {
                description: err.to_string().into(),
            });
        } else {
            ctx.span().set_status(Status::Ok);
        }
        ctx.span().end();
        self
    }
}
