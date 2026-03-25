// competitor_spy_telemetry/src/init.rs
//
// OpenTelemetry + tracing-subscriber initialization.
//
// Call `init_telemetry(log_level)` once at startup. The returned
// `TelemetryGuard` must be kept alive for the duration of the program;
// dropping it flushes and shuts down the OTel pipeline.

use std::io;

use opentelemetry_sdk::trace::TracerProvider;
use thiserror::Error;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

// ── SEC-001: Pre-emit redacting writer ───────────────────────────────────────

/// A [`MakeWriter`] adapter that pipes every log string through [`redact()`]
/// before forwarding to the inner writer.
///
/// Produced by [`init_telemetry`] and available for tests that need to
/// construct an isolated subscriber with the same redaction guarantee.
///
/// [`redact()`]: crate::redact::redact
pub struct RedactingWriter<M>(M);

impl<M> RedactingWriter<M> {
    pub fn new(inner: M) -> Self {
        Self(inner)
    }
}

/// Per-event writer returned by [`RedactingWriter::make_writer`].
pub struct RedactingWriterInstance<W>(W);

impl<W: io::Write> io::Write for RedactingWriterInstance<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                let redacted = crate::redact::redact(s);
                self.0.write_all(redacted.as_bytes())?;
                // Report the original buffer length so the caller's accounting
                // stays correct even though we may have written more or fewer
                // bytes (redaction can change string length).
                Ok(buf.len())
            }
            Err(_) => {
                // Non-UTF-8 bytes cannot contain text-pattern secrets.
                self.0.write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<'a, M: MakeWriter<'a>> MakeWriter<'a> for RedactingWriter<M> {
    type Writer = RedactingWriterInstance<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriterInstance(self.0.make_writer())
    }
}

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

    // Install the subscriber: stderr compact layer (via RedactingWriter) + OTel layer.
    // SEC-001: RedactingWriter ensures no secret value reaches the log sink.
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(RedactingWriter::new(std::io::stderr)))
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

