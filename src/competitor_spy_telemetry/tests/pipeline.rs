// pipeline.rs — T-037
// Integration test: verifies that the tracing log pipeline applies redact()
// as a pre-emit filter so no secret value ever appears in emitted log entries.
//
// FORMAL_SPEC.md §6.3:
//   "Secrets redacted before log emission by a pre-emit filter; no credential
//    value, API key, or token ever appears in any log entry."
// NFR traceability (FORMAL_SPEC.md line 529):
//   "secret-filter audit log test" — planned and named, never implemented.
//
// RED STATE — this test binary will NOT COMPILE until `RedactingWriter` is
// exported from `competitor_spy_telemetry::init`.  That type is introduced
// by the T-037 code fix.  The compile failure is intentional and isolated to
// this test binary; all other crate test binaries still run.
//
// GREEN STATE — once `RedactingWriter<W: io::Write>` is created in init.rs
// and re-exported, this binary compiles.  The assertion then passes, proving
// the wiring is in place and acting as a permanent regression guard.

use std::sync::{Arc, Mutex};

use tracing_subscriber::prelude::*;

// RED: `RedactingWriter` does not exist yet.
// Fix: add `pub struct RedactingWriter<W>(W);` to competitor_spy_telemetry/src/init.rs,
// implement `io::Write` by calling `redact()` on each buffer slice before forwarding,
// implement `MakeWriter<'_>` so it can be passed to `.with_writer(...)`,
// then re-export it: `pub use init::RedactingWriter;` in lib.rs.
use competitor_spy_telemetry::init::RedactingWriter;

// ── Captured output writer ────────────────────────────────────────────────────

/// A simple in-memory writer that captures all bytes written to it.
/// Used to inspect what the tracing fmt layer actually emits.
struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl tracing_subscriber::fmt::MakeWriter<'_> for CaptureWriter {
    type Writer = CaptureWriter;

    fn make_writer(&self) -> Self::Writer {
        CaptureWriter(Arc::clone(&self.0))
    }
}

// ── T-037 test ────────────────────────────────────────────────────────────────

/// SEC-001 / T-037 — FORMAL_SPEC.md §6.3
///
/// Verifies that tracing events containing known secret patterns are scrubbed
/// before reaching the output sink.  The subscriber is constructed using
/// `RedactingWriter` wrapping a `CaptureWriter`, mirroring the wiring that
/// `init_telemetry()` must apply.
///
/// Uses `with_default()` (not `try_init()`) to avoid polluting the global
/// subscriber for other test workers running in the same process.
#[test]
fn redact_filter_wired_into_log_pipeline() {
    let buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    // Build: fmt layer → RedactingWriter (applies redact()) → CaptureWriter.
    // This is the same layering that init_telemetry() must use after the fix.
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(RedactingWriter::new(CaptureWriter(Arc::clone(&buf)))),
    );

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Authorization: Bearer sk-FAKE-TOKEN-9999");
        tracing::info!("Sending adapter request with api_key=verysecretapikey999");
    });

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();

    assert!(
        !output.contains("sk-FAKE-TOKEN-9999"),
        "SEC-001: raw Bearer token present in log output.\n\
         RedactingWriter is not applied to the fmt layer writer.\n\
         Captured:\n{output}"
    );
    assert!(
        !output.contains("verysecretapikey999"),
        "SEC-001: raw API key present in log output.\n\
         Captured:\n{output}"
    );
    assert!(
        output.contains("[REDACTED]"),
        "SEC-001: [REDACTED] marker absent from log output.\n\
         RedactingWriter must call redact() on each buffer before forwarding.\n\
         Captured:\n{output}"
    );
}
