# Operations and Rollback Plan — Competitor Spy v1.0

---

## Normal Operation

**Run the tool:**
```
competitor-spy --industry <industry> --location <location> --radius <5|10|20|25|50>
```

**Standard run checklist:**
1. Confirm internet connectivity.
2. Set `CSPY_CREDENTIAL_PASSPHRASE` if Yelp or Google credentials are in the store.
3. Confirm `--output-dir` is writable (PDF output).
4. Run with `--log-level info` (default). Use `--log-level debug` for adapter-level diagnostics.

**Capture a full session log:**
```powershell
# Windows
.\scripts\capture_session.ps1

# Linux
bash scripts/capture_session.sh
```
Session logs land in `docs/evidence/sessions/`.

---

## Credential Management

- **First-time setup:** Run normally; prompted on stderr for missing API keys.
- **Rotate a key:** Delete the store file and re-enter on next run, or store file path:
  - Windows: `%APPDATA%\competitor-spy\credentials`
  - Linux: `~/.config/competitor-spy/credentials`
- **Passphrase rotation:** Delete the store file; all credentials will be re-prompted on next run.

---

## Rollback Triggers

- Binary produces wrong exit code for a known input.
- Report output is structurally invalid or missing required sections.
- Credential store is corrupted or refuses to decrypt.
- A new adapter or dependency introduced a regression (build red).

## Rollback Procedure

1. Identify last known-good commit: `git log --oneline`.
2. Create a recovery branch: `git checkout -b recovery/<date>`.
3. Revert only the bad commit(s): `git revert <commit-sha>` (non-destructive).
4. Run `cargo test --workspace` to confirm green.
5. Run a live test: canonical query against Amsterdam.
6. Document the rollback in `memory.md` and commit.

## Safety Rule

- Never use `git reset --hard` on a branch that has been shared or pushed.
- For rollback review: always run the full 198-test suite to confirm green before re-publishing.

---

## Known Binary Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Nominatim rate limiting (HTTP 429) | Low | Pacing [5, 15]s per request; single user; low request volume |
| Overpass overloaded (HTTP 503) | Low | Non-fatal; adapter skipped; run still produces reports |
| Yelp/Google API key expiry | Low | ADAPTER_CONFIG_MISSING in footer; user notified; re-enter key |
| PDF write permission error | Low | Warning only; terminal output still produced; exit 0 |
| Linux binary untested | Medium | See `docs/evidence/environment-matrix.md`; validate before production use |
