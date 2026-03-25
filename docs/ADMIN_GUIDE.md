# Admin Guide — Competitor Spy

Configuration reference, credential management, adapter setup, and troubleshooting. For usage, see the [README](../README.md). For building from source, see the [Developer Guide](DEVELOPER_GUIDE.md).

---

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `CSPY_CREDENTIAL_PASSPHRASE` | Required when using Yelp or Google Places | Passphrase to unlock the encrypted credential store. Must be set before running any `credentials` subcommand or any search that needs stored keys. |
| `CSPY_PACING_SEED` | No | Integer seed for deterministic request pacing. Overridden by the `--pacing-seed` CLI flag if both are set. |

Set for the current shell session:

```bash
export CSPY_CREDENTIAL_PASSPHRASE="your-passphrase"   # Linux / Git Bash
$env:CSPY_CREDENTIAL_PASSPHRASE = "your-passphrase"   # PowerShell
```

Never pass the passphrase as a CLI argument — it would appear in process lists and shell history.

---

## Credential store

API keys for Yelp and Google Places are stored encrypted using the [age](https://age-encryption.org/) format with passphrase-based encryption.

**Store location:**

| OS | Path |
|---|---|
| Windows | `%APPDATA%\competitor-spy\credentials` |
| Linux | `~/.config/competitor-spy/credentials` |

The store is created automatically on first use. If the passphrase changes, the store must be re-created (delete the file and re-add keys).

### credentials subcommand

```bash
# store a key (key is read from stdin, no echo)
competitor-spy credentials set yelp
competitor-spy credentials set google_places

# check which keys are stored
competitor-spy credentials list

# remove a key
competitor-spy credentials delete yelp
```

`CSPY_CREDENTIAL_PASSPHRASE` must be set before running any of the above.

---

## Adapter configuration

| Adapter | Key name | Credentials required | Notes |
|---|---|---|---|
| OSM / Overpass | — | None | Public API. Occasionally returns HTTP 503 (transient). |
| Nominatim | — | None | Used for geocoding and as a search source. Rate-limited; one request per second enforced internally. |
| Yelp Fusion | `yelp` | Yes | Requires a Yelp Fusion API key. |
| Google Places | `google_places` | Yes | Requires a Google Places API key with Places API enabled. |

Adapters without stored credentials show `ADAPTER_CONFIG_MISSING` in the report footer. This is non-fatal (exit code 0); results from other sources are still displayed.

---

## Report output path

By default, PDFs are written to a `reports/` directory anchored to the location of the `competitor-spy` binary (three directory levels up from `target/release/`). This means the PDF always lands in the project root `reports/` folder regardless of the working directory when the binary is invoked.

Override with `--output-dir`:

```bash
./competitor-spy --industry "yoga" --location "Vienna" --radius 10 --output-dir /tmp/my-reports
```

**PDF filename format:** `{industry}_{location}_{radius}km_{YYYYMMDD}_{HHMM}.pdf`  
Both `{industry}` and `{location}` are slugified to alphanumeric lowercase, capped at 10 characters.  
Example: `pilates_stpoelten_10km_20260325_0941.pdf`

---

## Log levels

Set with `--log-level` (default: `info`):

| Level | Output |
|---|---|
| `error` | Fatal errors only |
| `warn` | Adapter failures, enrichment warnings |
| `info` | Normal run progress, PDF path, enrichment coverage |
| `debug` | HTTP requests, geocoding candidates, deduplication detail |
| `trace` | Full adapter response payloads |

---

## Enrichment (V3 website scraping)

After collecting competitor profiles, the tool fetches each competitor's website (where available) and extracts:

- Pricing / membership tiers
- Lesson / class types
- Schedule information
- Testimonials
- Class descriptions

**Coverage threshold:** 60% of competitors must yield at least one enrichable field for the run to report "good coverage". The coverage percentage is shown in the report footer.

**Enrichment flags:**

| Flag | Effect |
|---|---|
| `--no-enrichment` | Skip website fetching entirely; faster but no enrichment data |
| `--enrichment-timeout <secs>` | Per-request HTTP timeout (default: 15s) |
| `--allow-insecure-tls` | Accept self-signed or expired TLS certificates |

---

## Troubleshooting

| Symptom | Likely cause | Resolution |
|---|---|---|
| `error: CSPY_CREDENTIAL_PASSPHRASE environment variable is not set` | Env var not exported | Set `CSPY_CREDENTIAL_PASSPHRASE` before running |
| `error: failed to open credential store` | Wrong passphrase, or store corrupted | Verify passphrase; if corrupted, delete store file and re-add keys |
| `yelp: ADAPTER_CONFIG_MISSING` | No Yelp key stored | Run `credentials set yelp` |
| `google_places: ADAPTER_CONFIG_MISSING` | No Google key stored | Run `credentials set google_places` |
| `osm_overpass: HTTP_5XX` | Overpass API temporarily overloaded | Transient — retry in a few minutes |
| `GEO_NO_RESULT` — geocoding failure | Location string not recognised | Use city + country format, e.g. `"Berlin, Germany"` |
| PDF written to wrong folder | Binary invoked from unexpected CWD | Use `--output-dir` to set an explicit path |
| No competitors found | All adapters failed or returned zero results | Check failed-sources footer; try broader industry term or larger radius |

Full failure runbook: `governance/RUNBOOK_KNOWN_FAILURES.md`
