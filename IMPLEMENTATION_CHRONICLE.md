# Implementation Chronicle

## Entry CHR-SNK-001

- Task: T-001
- Requirement: FR-001, FR-006, NFR-004
- Decision: enforce startup minimum size check before any curses init.
- Why: guarantees required print+log+exit behavior even when rendering cannot start.
- Evidence: `startup_size_check` and `log_size_crash` implementation.

## Entry CHR-SNK-002

- Task: T-002
- Requirement: FR-001
- Decision: center head at `fixed_width // 2`, `fixed_height // 2`; pre-start loop waits for any key.
- Why: exact startup placement and wait behavior from brief.
- Evidence: pre-start state in `run_game`.

## Entry CHR-SNK-003

- Task: T-003
- Requirement: FR-002
- Decision: constant-tick movement with direction updates from arrow keys only.
- Why: player controls direction only; no speed control.
- Evidence: `KEY_TO_DIR`, movement loop, fixed `TICK_SECONDS`.

## Entry CHR-SNK-004

- Task: T-004
- Requirement: FR-003
- Decision: when apple eaten, insert old head behind new head: `[new_head, old_head] + rest`.
- Why: implements growth directly behind head exactly as specified.
- Evidence: apple-eat branch in game loop.

## Entry CHR-SNK-005

- Task: T-005
- Requirement: FR-004
- Decision: out-of-bounds and body-hit end game immediately.
- Why: strict wall/self collision loss semantics, no wraparound.
- Evidence: collision checks before snake update finalization.

## Entry CHR-SNK-006

- Task: T-006
- Requirement: FR-005
- Decision: win on full board after final apple consumption.
- Why: exact explicit win contract from brief.
- Evidence: `if len(snake) == fixed_width * fixed_height` after growth.

## Entry CHR-SNK-007

- Task: T-007
- Requirement: FR-007
- Decision: only write leaderboard when `score > previous_high`.
- Why: brief prohibits non-record writes and prompts.
- Evidence: conditional name prompt and write path in `main`.

## Entry CHR-SNK-008

- Task: T-008
- Requirement: runtime terminal support in Git Bash
- Decision: Bash launcher uses `winpty` on MSYS if available.
- Why: enables curses compatibility under Git Bash on Windows.
- Evidence: `run_snake.sh` conditional execution path.

## Stage 4 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Build complete with chronicle links for all tasks.

---

# Competitor Spy Implementation Chronicle

## Entry CHR-CSPY-000

- Task: T-000
- Date: 2026-03-21
- Requirement: CSPY-PLAN-001
- Spec ref: FORMAL_SPEC.md §Q3-ARCH-01 (layered architecture)

### Decision: 6-crate workspace with compiler-enforced layer isolation

All 6 roles (domain, adapters, output, credentials, telemetry, cli) are separate Cargo crates in a single workspace. The dependency graph is:

```
competitor_spy_domain          (no I/O, no async, no rendering deps)
  ↑
competitor_spy_adapters        (async HTTP; depends on domain)
competitor_spy_credentials     (sync file I/O; depends on nothing except age/serde)
competitor_spy_telemetry       (async OTel; depends on nothing domain-specific)
competitor_spy_output          (sync rendering; depends on domain)
  ↑
competitor_spy_cli             (entry point; depends on all 5 above)
```

Why separate crates: If domain accidentally imports tokio or reqwest, the compiler rejects it. No lint rule or code-review custom needed — the dep graph is the enforcement. Adapter crates cannot import output or credentials, enforced by the same mechanism.

### Decision: resolver = "2" and workspace-level dependency pinning

`[workspace.dependencies]` declares every shared version once. All crate `Cargo.toml` files use `{ workspace = true }` to inherit. This prevents version drift between crates and makes audits and upgrades a single-location change.

Key pins:
- tokio 1, features = full (gives all runtime primitives; trimmed in specific crates if needed)
- reqwest 0.12, no-default-features, features = [json, rustls-tls] (avoids openssl C dep on Windows)
- age 0.10 (passphrase-based encryption for credential store)
- opentelemetry 0.27 + tracing-opentelemetry 0.28 (compatible pair; OTel 0.x API stability)
- clap 4, features = [derive] (ergonomic CLI; derive macro means no manual arg builder)
- wiremock 0.6 (dev-only; HTTP mock server for adapter tests)

### Decision: CSPY_STATE_LOG env-var pattern for capture scripts

Capture scripts (`capture_session.sh` and `capture_session.ps1`) write all stdout + stderr to a session log file at path given by env var `CSPY_STATE_LOG`. If the var is not set, a default timestamped path is constructed. Output is simultaneously teed to the terminal. The binary is invoked with `--log-level trace` to ensure maximum diagnostic data. Exit code is propagated faithfully (PIPESTATUS on bash; `$process.ExitCode` on PS).

### Evidence

- `cargo build` output: 335+ packages downloaded, all 6 crates compiled, zero errors.
- Binary confirmed: `target\debug\competitor-spy.exe` (Test-Path = True).
- All 6 crates: `cargo build --quiet` returns no warnings relevant to stub structure.
- TASK_LIST.md: T-000 status = DONE, evidence-date = 2026-03-21.
