# Operational Notes and Rollback Plan

## Operational Notes

- Governance artifacts are source of truth and must be updated in stage order.
- Required logs are `prompts.md` and `memory.md`.
- Stage transitions require explicit approval and a stage-completion commit.
- Verify-stage traceability gaps are blockers and must be resolved before closure.

## Rollback Triggers

- A stage was advanced without explicit approval.
- A required artifact was skipped or corrupted.
- Traceability links are missing for done items.
- Brownfield confidence gate was bypassed.

## Rollback Procedure

1. Identify last valid stage-completion commit using `git log --oneline`.
2. Create a recovery branch from current HEAD.
3. Revert only the invalid commit(s) using non-destructive `git revert`.
4. Re-run the affected stage checklist.
5. Record rollback reason and recovery actions in `memory.md`.
6. Re-request stage approval and commit completion again.

## Safety Rule

- Do not use destructive history rewrite for shared history during normal governance recovery.
