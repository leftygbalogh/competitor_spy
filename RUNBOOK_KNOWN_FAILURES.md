# Runbook for Known Failure Scenarios

## Scenario 1: Stage Transition Denied

- Symptom: Attempt to move to next stage is blocked.
- Expected action:
  1. Record denial reason in active stage artifact and `memory.md`.
  2. Resolve missing approval or missing stage-completion commit.
  3. Re-run stage gate checks.

## Scenario 2: Verify Traceability Gap

- Symptom: FR/NFR link to task/test/chronicle is missing.
- Expected action:
  1. Mark gap as blocker.
  2. Update `FORMAL_SPEC.md` traceability matrix.
  3. Update `TASK_LIST.md` or `IMPLEMENTATION_CHRONICLE.md` links.
  4. Re-run Verify checklist.

## Scenario 3: Brownfield Confidence Below Threshold

- Symptom: Baseline evidence incomplete or confidence low.
- Expected action:
  1. Block new feature commitments.
  2. Execute allowed actions from Brownfield decision table.
  3. Re-assess confidence and document in `memory.md`.

## Scenario 4: Logging Drift

- Symptom: Missing or stale prompt/status snapshots.
- Expected action:
  1. Append missing user prompt to `prompts.md`.
  2. Add current status snapshot to `memory.md`.
  3. Commit checkpoint before further stage actions.
