# Getting Started — Competitor Spy v1.0

Competitor Spy is a local OSINT competitor-intelligence CLI tool written in Rust. It geocodes a location, queries public business data sources, normalises and ranks results, and outputs a terminal table and a PDF report.

---

## Prerequisites

- **Rust toolchain** — stable, 1.79 or later. Install via [rustup.rs](https://rustup.rs).
- **Windows 11 x86_64** or **Linux x86_64** (see `docs/evidence/environment-matrix.md` for support status).
- Internet access for geocoding (Nominatim) and data sources.
- Optional: Yelp Fusion API key and/or Google Places API key for richer results.

---

## Build

```powershell
# Windows (PowerShell)
cargo build --release -p competitor_spy_cli
# Binary: target\release\competitor-spy.exe

# Linux (Bash)
cargo build --release -p competitor_spy_cli
# Binary: target/release/competitor-spy
```

All workspace tests must pass before use:

```powershell
cargo test --workspace
# Expected: 198 tests, 0 failures (193 unit + 5 acceptance)
```

---

## Basic Usage

```
competitor-spy \
  --industry "yoga studio" \
  --location "Amsterdam, Netherlands" \
  --radius 10
```

This will:
1. Geocode "Amsterdam, Netherlands" via Nominatim.
2. Query all configured data adapters (OSM/Overpass, Nominatim, Yelp, Google Places) within 10 km.
3. Normalise, deduplicate, and rank results.
4. Print a terminal table and save a PDF to the current directory.

**Available flags:**

| Flag | Required | Default | Description |
|---|---|---|---|
| `--industry` | Yes | — | Business type to search for |
| `--location` | Yes | — | Location string (geocoded) |
| `--radius` | Yes | — | Search radius: 5, 10, 20, 25, or 50 km |
| `--output-dir` | No | `.` (cwd) | Directory to save PDF |
| `--no-pdf` | No | off | Skip PDF generation |
| `--log-level` | No | `info` | Trace, debug, info, warn, or error |
| `--pacing-seed` | No | random | Deterministic pacing seed (testing/debug) |

**Exit codes:** 0 = success (even if some sources fail). 1 = fatal error (bad args, geocoding failure, terminal render failure).

---

## API Credentials (Optional)

Yelp and Google Places require API keys for results. On first run, if a key is not stored, you will be prompted on stderr with echo disabled.

Keys are stored encrypted (age, passphrase-based) at:
- Windows: `%APPDATA%\competitor-spy\credentials`
- Linux: `~/.config/competitor-spy/credentials`

Set passphrase via environment variable (never on the command line):
```powershell
$env:CSPY_CREDENTIAL_PASSPHRASE = "your-passphrase"
```

---

## Capturing a Session (for Diagnostics)

```powershell
# Windows
.\scripts\capture_session.ps1

# Linux
bash scripts/capture_session.sh
```

Session logs are saved to `docs/evidence/sessions/` with naming `session_YYYYMMDD_HHMMSS_<label>.log`.

---

## PDF Output

PDF files are saved to `--output-dir` (default: working directory). Filename format:
```
competitor_spy_report_YYYYMMDD_HHMMSS_UTC.pdf
```

---

## Understanding the Report

- **Rank** — order by distance ascending, then keyword-relevance descending, then name alphabetically.
- **Keyword%** — token overlap between competitor categories and your industry query, 0–100%.
- **Visibility%** — composite of profile completeness and review count, 0–100%.
- **`--`** — field absent from all sources for this competitor.
- **Failed sources footer** — sources that returned 4xx/5xx/timeout/parse errors. Failures are non-fatal; results from other sources are still shown.

---

## Troubleshooting

See `RUNBOOK_KNOWN_FAILURES.md` for specific failure scenarios and recovery steps.
