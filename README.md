# Competitor Spy

A local OSINT competitor-intelligence CLI tool. Give it an industry and a location; it queries public business data sources, enriches results by scraping competitor websites, and produces a terminal report and a PDF.

## Quick Start

```bash
# set your credential passphrase (required when API keys are stored)
export CSPY_CREDENTIAL_PASSPHRASE="your-passphrase"   # Linux / Git Bash
$env:CSPY_CREDENTIAL_PASSPHRASE = "your-passphrase"   # PowerShell

# run a search
./competitor-spy --industry "yoga studio" --location "Vienna, Austria" --radius 10
```

A PDF is written to `reports/` inside the project folder. Terminal output is printed immediately.

---

## Usage

```
competitor-spy --industry <TEXT> --location <TEXT> --radius <KM> [OPTIONS]
```

### Required flags

| Flag | Description |
|---|---|
| `--industry` | Business type to search for (e.g. `"pilates studio"`) |
| `--location` | Location string, geocoded via Nominatim (e.g. `"St Pölten, Austria"`) |
| `--radius` | Search radius in km — must be one of: `5`, `10`, `20`, `25`, `50` |

### Optional flags

| Flag | Default | Description |
|---|---|---|
| `--output-dir` | `reports/` in project root | Directory to write the PDF |
| `--no-pdf` | off | Skip PDF, terminal output only |
| `--detail` | on | Show enrichment fields (pricing, lesson types, schedule, etc.) |
| `--no-enrichment` | off | Skip website scraping entirely |
| `--enrichment-timeout` | `15` | HTTP timeout in seconds for website enrichment requests |
| `--allow-insecure-tls` | off | Accept invalid TLS certificates when fetching competitor sites |
| `--log-level` | `info` | Verbosity: `trace`, `debug`, `info`, `warn`, `error` |
| `--pacing-seed` | random | Deterministic request pacing seed (useful for testing) |

### Subcommand: credentials

Manage encrypted API keys for Yelp and Google Places.  
`CSPY_CREDENTIAL_PASSPHRASE` must be set before running any credentials command.

```bash
competitor-spy credentials set yelp           # store API key (prompted, no echo)
competitor-spy credentials set google_places  # store API key
competitor-spy credentials list               # show which keys are set
competitor-spy credentials delete yelp        # remove a key
```

---

## PDF output

PDFs are saved to `reports/` in the project directory by default. You can override with `--output-dir`.

Filename format: `{industry}_{location}_{radius}km_{YYYYMMDD}_{HHMM}.pdf`  
Example: `pilates_stpoelten_10km_20260325_0941.pdf`

---

## Data sources

| Source | Credentials required |
|---|---|
| OpenStreetMap / Overpass | None |
| Nominatim (geocoding + search) | None |
| Yelp Fusion | Yes — `yelp` API key |
| Google Places | Yes — `google_places` API key |

Results from all available sources are merged, deduplicated, and ranked by distance. Sources without credentials are skipped silently. Source failures are listed in the report footer and are non-fatal (exit code 0).

---

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success — even if individual sources failed |
| `1` | Fatal error — bad arguments, geocoding failure |

---

## Further reading

- [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) — building from source, running tests
- [docs/ADMIN_GUIDE.md](docs/ADMIN_GUIDE.md) — environment variables, credential store, adapter config, troubleshooting
