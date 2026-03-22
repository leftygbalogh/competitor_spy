# ADR-005: Credential Encryption via `age` Crate

**Date:** 2026-03-21
**Status:** Accepted
**Spec reference:** FORMAL_SPEC.md §7, Decision 5

---

## Context

Competitor Spy accepts user-supplied API keys for Yelp and Google Places. These keys must be stored persistently so the user is not prompted on every run. Storing them in plaintext on disk is unacceptable (GDPR alignment, security baseline).

## Decision

Use the `age` crate (Rust implementation of the `age` encryption format) with passphrase-based encryption. The passphrase is supplied by the user via the `CSPY_CREDENTIAL_PASSPHRASE` environment variable or prompted on first use.

Store location:
- Windows: `%APPDATA%\competitor-spy\credentials`
- Linux: `~/.config/competitor-spy/credentials`

## Rationale

- **Audited format.** The `age` format is modern, minimal, and publicly specified. The Rust `age` crate is widely used.
- **User-controlled.** The passphrase belongs to the user. The tool never stores or transmits the passphrase. Rotating the passphrase means deleting the store and re-entering credentials.
- **No OS dependency.** Works across Windows and Linux without OS keychain APIs.
- **Simple schema.** The encrypted payload is a serialized `HashMap<adapter_id, credential_bytes>`. No migration complexity in v1.

## Consequences

- If the user forgets their passphrase, they must delete the credential store and re-enter all keys.
- The passphrase is never written to a log, file, or stdout. Supplied via environment variable (`CSPY_CREDENTIAL_PASSPHRASE`) to avoid it appearing in shell history.
- Credential decryption happens once per run, in-memory. The decrypted value is not persisted anywhere.
- Secrets are redacted from all OpenTelemetry log events by a pre-emit filter in `competitor_spy_telemetry`.

## Alternatives Rejected

- **OS keychain (`keyring` crate)** — cross-platform reliability issues in v1; requires platform-specific setup on Linux. Rejected.
- **Plaintext file** — fails GDPR alignment and security baseline. Rejected.
- **Environment variable only** — convenient for CI but not suitable for interactive use; user would have to set and manage API keys externally on every session. Rejected for primary credential storage (env var used for passphrase only).
