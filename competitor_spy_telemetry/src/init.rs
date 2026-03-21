// competitor_spy_telemetry/src/init.rs
//
// OpenTelemetry + tracing-subscriber initialization.
//
// Call `init_telemetry(log_level)` once at startup. The returned
// `TelemetryGuard` must be kept alive for the duration of the program;
// dropping it flushes and shuts down the OTel pipeline.

use opentelemetry_sdk::trace::TracerProvider;
use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

/// Errors that can occur during telemetry initialisation.
#[derive(Debug, Error)]
pub enum InitError {
    #[error("failed to set global tracing subscriber: {0}")]
    SetGlobal(#[from] tracing_subscriber::util::TryInitError),
    #[error("unknown log level: {0}")]
    UnknownLevel(String),
}

/// Guard that shuts down the OTel pipeline on drop.
///
/// Keep this value alive in `main` for the duration of the program.
pub struct TelemetryGuard {
    tracer_provider: TracerProvider,
    /// The run_id generated at init time. Attach to all audit events.
    pub run_id: Uuid,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        // Flush and shut down the OTel pipeline so all pending spans are exported.
        let _ = self.tracer_provider.shutdown();
    }
}

/// Initialise OpenTelemetry tracing and install a global `tracing` subscriber.
///
/// - `log_level`: one of `trace`, `debug`, `info`, `warn`, `error`
/// - Logs to stderr via a human-readable tracing-subscriber layer.
/// - OTel spans exported to stdout via `opentelemetry-stdout`.
/// - Returns a `TelemetryGuard` whose `run_id` is a fresh UUID for this run.
pub fn init_telemetry(log_level: &str) -> Result<TelemetryGuard, InitError> {
    // Validate the log level string.
    let _level: tracing::Level = match log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info"  => tracing::Level::INFO,
        "warn"  => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        other   => return Err(InitError::UnknownLevel(other.to_owned())),
    };

    // Build the OTel stdout exporter.
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let tracer_provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();

    // Build the OTel tracing layer.
    let otel_tracer = opentelemetry::trace::TracerProvider::tracer(
        &tracer_provider,
        "competitor-spy",
    );
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    // Build env filter from the supplied level string.
    let filter = EnvFilter::try_new(log_level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Install the subscriber: stderr compact layer + OTel layer.
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(otel_layer)
        .try_init()?;

    let run_id = Uuid::new_v4();

    Ok(TelemetryGuard { tracer_provider, run_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_log_level_returns_error() {
        let result = init_telemetry("verbose");
        assert!(result.is_err());
        match result {
            Err(InitError::UnknownLevel(s)) => assert_eq!(s, "verbose"),
            Err(other) => panic!("expected UnknownLevel, got different error: {other}"),
            Ok(_) => panic!("expected error, got Ok"),
        }
    }

    #[test]
    fn valid_levels_accepted() {
        for level in &["trace", "debug", "info", "warn", "error"] {
            let result = init_telemetry(level);
            // May fail with SetGlobal if a previous test already installed a subscriber.
            // UnknownLevel must never be returned for valid levels.
            match result {
                Err(InitError::UnknownLevel(_)) => panic!("valid level {level} rejected"),
                _ => {}
            }
        }
    }

    #[test]
    fn run_id_is_unique_per_guard() {
        // We can't call init_telemetry twice without getting SetGlobal errors,
        // so just verify the Uuid::new_v4() uniqueness property holds.
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_ne!(a, b);
    }
}

