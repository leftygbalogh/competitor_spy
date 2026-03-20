# Post-Release Monitoring Plan

## Monitoring Window

- Immediate: first working session after release.
- Short-term: first 3 project onboardings using this template.
- Ongoing: every stage-gate cycle in each downstream project.

## Signals to Monitor

- Number of stage-transition denials and reasons.
- Number of traceability blockers found at Verify stage.
- Number of template changes requested per downstream project.
- Number of missing prompt/memory log events.

## Alert Thresholds

- Critical: any stage transition without explicit approval.
- High: any unresolved Verify traceability blocker.
- Medium: repeated missing logging entries within one cycle.
- Low: template friction signals (frequent clarifying governance edits).

## Alerting Route

1. Record issue in `memory.md` with blocker, impact, and next action.
2. Add prompt context to `prompts.md` if user-triggered.
3. Stop stage transition until blocker is resolved.
4. Escalate to decision owner (Lefty) for approval decision.
