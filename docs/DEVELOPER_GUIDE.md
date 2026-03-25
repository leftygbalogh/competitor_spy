# Developer Guide — Competitor Spy

This guide covers building from source, running the test suite, project structure, and diagnostics. For usage, see the [README](../README.md). For configuration and credentials, see the [Admin Guide](ADMIN_GUIDE.md).

---

## Prerequisites

- **Rust toolchain** — stable, 1.79 or later. Install via [rustup.rs](https://rustup.rs).
- **cargo-audit** — dependency CVE scanner. Install once: `cargo install cargo-audit`.
- **Windows 11 x86_64** or **Linux x86_64** — see `docs/evidence/environment-matrix.md` for full support matrix.
- Internet access for geocoding (Nominatim) and data sources during live tests.

---

## Build

```bash
# release binary
cargo build --release -p competitor_spy_cli

# Windows binary:  target/release/competitor-spy.exe
# Linux binary:    target/release/competitor-spy
```

The binary is self-contained — no runtime dependencies, no config files required.

---

## Test suite

```bash
cargo test --workspace
# 261 tests: unit + acceptance. All must pass on a green build.
```

Scan dependencies for known CVEs before any release:

```bash
cargo audit
# Zero unfixed advisories required. Install once: cargo install cargo-audit
```

The acceptance tests (`src/competitor_spy_cli/tests/acceptance.rs`) use local stub URLs and do not require internet access or real API credentials.

Run a single crate's tests:

```bash
cargo test -p competitor_spy_domain
cargo test -p competitor_spy_adapters
cargo test -p competitor_spy_output
cargo test -p competitor_spy_credentials
cargo test -p competitor_spy_cli
```

---

## Workspace structure

| Crate | Purpose |
|---|---|
| `competitor_spy_domain` | Core domain types: `SearchRun`, `Competitor`, `Enrichment`, ranking, deduplication |
| `competitor_spy_adapters` | Source adapters (OSM/Overpass, Nominatim, Yelp, Google Places) + website enrichment extractors |
| `competitor_spy_output` | Terminal renderer and PDF renderer |
| `competitor_spy_credentials` | Encrypted credential store (age, passphrase-based) |
| `competitor_spy_telemetry` | Tracing / log initialisation |
| `competitor_spy_cli` | Binary entry point — argument parsing, runner orchestration |

---

## Live E2E run (manual)

```bash
export CSPY_CREDENTIAL_PASSPHRASE="your-passphrase"
./target/release/competitor-spy --industry "pilates" --location "Vienna, Austria" --radius 10
```

Expected: terminal output + PDF in `reports/`. Check the failed-sources footer for any adapter errors — OSM Overpass occasionally returns HTTP 503 (transient, non-fatal).

---

## Session capture (diagnostics)

```bash
# Linux / Git Bash
bash scripts/capture_session.sh

# PowerShell
.\scripts\capture_session.ps1
```

Saves a timestamped log to `docs/evidence/sessions/session_YYYYMMDD_HHMMSS_<label>.log`.

---

## Key files

| File | Purpose |
|---|---|
| `src/competitor_spy_cli/src/runner.rs` | Core run orchestration, credential resolution, output path logic |
| `src/competitor_spy_cli/src/main.rs` | CLI argument definitions (clap), subcommand routing |
| `src/competitor_spy_adapters/src/web_enricher.rs` | Website enrichment orchestrator |
| `src/competitor_spy_output/src/pdf.rs` | PDF rendering + filename format |
| `src/competitor_spy_cli/tests/acceptance.rs` | Acceptance test suite |

---

## Governance and architecture

- Architecture decisions: `docs/adr/`
- Formal spec: `governance/FORMAL_SPEC.md`
- Task list: `governance/TASK_LIST.md`
- Implementation chronicle: `governance/IMPLEMENTATION_CHRONICLE.md`
