// competitor_spy_telemetry
//
// Async telemetry crate. Initialises OpenTelemetry tracing, applies a
// pre-emit secret redaction filter, and exposes structured event helpers.
// No credential values, API keys, or tokens ever reach the log sink.

pub mod init;
pub mod redact;

// SEC-001: re-export so tests and downstream crates can assert on the full
// wiring without depending on the internal module path.
pub use init::RedactingWriter;
