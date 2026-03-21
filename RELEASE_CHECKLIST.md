# Release Checklist

## Stage 5 Verify Results

- [x] Syntax check passes: `python -m py_compile snake.py`
- [x] Startup behavior verified (center head, random direction, press-any-key wait)
- [x] Movement and collisions verified
- [x] Apple/spawn/growth/score verified
- [x] Runtime resize-too-small path implemented with print+log+exit
- [x] Leaderboard policy verified (new-high-only)
- [x] No extra features/configuration added

## Stage 6 Release

- [x] Deliverables present: `snake.py`, `run_snake.sh`
- [x] Crash log contract implemented: `snake.log`
- [x] Leaderboard contract implemented: `leaderboard.csv`
- [x] Governance artifacts updated for stages 1-6
- [x] Agent feedback file updated
- [ ] Push target verified immediately before publish: `git remote -v` captured in evidence
- [ ] Intended remote and branch explicitly stated and approved before push
- [ ] Release remote proof snapshot saved: `docs/evidence/release-remote-proof.md`

## Stage 5 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Verification complete; no open blockers.

## Stage 6 Approval

- Approved by: Team Lead Agent (delegated)
- Approval date: 2026-03-20
- Notes: Release complete for this project iteration.
