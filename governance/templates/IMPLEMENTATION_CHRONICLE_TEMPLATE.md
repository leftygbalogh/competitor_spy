# Implementation Chronicle Template

## 1. Chronicle Metadata

- Chronicle ID:
- Source task ID:
- Source spec sections:
- Source requirements:
- Module / component name:
- Implementation language:
- Author:
- Date:
- Status: Draft | Final

## 2. Intent to Implementation Mapping

- What this unit implements from the behavioral spec:
- What must remain functionally equivalent across languages:
- What is intentionally language-specific in this implementation:

## 3. Implementation Decisions

- Data structures chosen and why:
- Algorithms chosen and why:
- Control-flow structure chosen and why:
- Boundary and interface decisions:
- Error-handling strategy:
- Performance or memory trade-offs accepted:
- File map (concrete files changed/created):
- Public symbols introduced/changed:
- Signature snapshot (functions/classes/types and key fields):

## 4. Alternatives Considered

- Alternative 1:
- Why rejected:
- Alternative 2:
- Why rejected:

## 5. Derived Invariants and Constraints

- Invariant 1:
- Invariant 2:
- Constraints inherited from the spec:
- Additional implementation constraints introduced:
- Boundary behavior and terminal/end-state rules:

## 6. Divergences and Clarifications

- Where the spec was ambiguous:
- How the ambiguity was resolved in code:
- Any controlled divergence from the spec:
- Follow-up needed in the spec or task list:

## 7. Testing Notes

- Unit tests added:
- Integration tests added:
- Property-based tests added:
- Edge cases covered:
- Failure modes exercised:
- Runtime/orchestration branch evidence covered:
- Golden input/output examples captured:

## 8. Reconstruction Notes

- If the code were lost, what another implementer must know to rebuild this unit faithfully:
- Order of implementation steps that mattered:
- Non-obvious pitfalls discovered during implementation:
- What not to change without updating the behavioral spec:
- Side effects and external touchpoints (files/network/stdout/runtime context):

## 9. Known Limitations

- Limitation 1:
- Reason accepted:
- Revisit trigger:

## 10. Approval / Review

- Reviewed by:
- Review date:
- Notes:
- Pair-programming references (if applicable):
  - Session log path:
  - Driver role(s):
  - Navigator/reviewer role(s):
  - Key disagreement and resolution summary:

## 11. Reconstruction Bundle (required at Stage 4 close for major modules)

- Source tree manifest relevant to this module:
- Import/dependency graph expectations:
- Recommended reconstruction order:
- Post-rebuild validation commands:
- Expected validation outputs/pass criteria: