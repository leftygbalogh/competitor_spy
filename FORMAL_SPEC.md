# Formal Specification

## 1. Specification Metadata

- Spec ID: FSP-AGT-001
- Version: 1.0
- Project mode: Greenfield
- Source brief: PROJECT_BRIEF.md
- Status: Approved for Stage 2
- Author: GitHub Copilot
- Reviewers: Lefty
- Active Q3 modules: None

## 1.1 Mode Constraints

Two types of projects, two different sets of rules:

**If new build (Greenfield) — like building a snake game from scratch:**
- Start clean. No need to preserve old behavior that doesn't exist yet.
- Pick the right design up front: clean game loop, clear input handling, testable collision rules.
- Define how the game should behave, then implement it.
- Keep first version small and working before adding features.

**If refactor or rewrite (Brownfield) — like improving an existing snake game:**
- Your first job is to understand what the current game actually does (not what docs say it does).
- Lock that existing behavior with characterization tests before touching any code.
- Change things only in tiny steps, checking nothing broke after each one.
- If the old game had weird food spawn logic, keep it unless Lefty explicitly approves changing it.

**For this project:**
- This project is Greenfield. Brownfield rules are not active here.
- The Brownfield section is kept as a ready-made rulebook for future projects that are rewrites or refactors, so teams do not have to invent those rules from scratch later.

## 1.2 Behavioral Specification Approach

Default methods used:
- Statechart coverage: yes
- Contract coverage: yes
- Decision table coverage: yes
- Escalation to mathematical verification: No

Statechart coverage (core flow):
- States:
  - S1 Idle
  - S2 Discover
  - S3 Specify
  - S4 Plan
  - S5 Build
  - S6 Verify
  - S7 Release
  - S8 Blocked
- Key transitions:
  - Idle -> Discover when a new project starts
  - Discover -> Specify only after explicit approval
  - Any active stage -> Blocked when DoR/DoD or missing artifact violations occur
  - Blocked -> previous active stage after violation resolved

Contract coverage examples:
- Operation: RequestStageTransition(currentStage, nextStage)
  - Pre: current stage done criteria satisfied
  - Pre: explicit user yes recorded
  - Post: transition granted and stage record updated
  - Error: transition denied with missing-criteria list
- Operation: MarkTaskDone(taskId)
  - Pre: universal DoD satisfied
  - Pre: chronicle entry linked
  - Post: status set to Done and traceability updated
  - Error: task remains In progress with blocking reasons

Decision table coverage examples:
- Area 1: Can work move to next stage?
- Area 2: Can a Brownfield feature be promised?
- Area 3: Can a governance change be merged?

## 2. Scope

- In scope:
  - Stage-gated governance workflow with explicit approvals
  - Universal DoR/DoD enforcement model
  - Prompt and memory logging requirements
  - Basic task status tracking (To do, In progress, Done)
  - Cross-agent clarification before escalation
  - Brownfield uncertainty and parity safeguards as reusable policy
- Out of scope:
  - One-click multi-platform CI/CD governance packs
  - Heavy compliance model unless explicitly activated by project trigger
  - Deep non-Rust/Python language-specific governance packs
  - Single-step production code generation workflows
  - Dashboard-heavy governance UI

## 3. Domain Model

- Ubiquitous language:
  - Stage, Gate, DoR, DoD, Task, Chronicle, Approval, Escalation, Milestone Commit
- Core entities:
  - ProjectBrief, FormalSpec, TaskList, ChronicleEntry, PromptLogEntry, MemorySnapshot
- Value objects:
  - StageName, ApprovalRecord, TraceabilityLink, RiskItem
- Aggregates:
  - GovernanceProject (root)
- Domain services:
  - StageGateEvaluator
  - TraceabilityMapper
  - BrownfieldRiskGate

Example:
- If a task is marked Done but has no chronicle link, StageGateEvaluator returns Blocked.

## 4. Functional Behavior

### FR-001 Template startup behavior
- Preconditions:
  - Governance folder exists in project
  - No project brief exists yet (clean start)
- Trigger:
  - Agent is asked to start work on a project
- Expected behavior:
  - Agent reads governance files in required order
  - Agent has no knowledge of the project yet — it only knows the governance rules
  - Agent asks: "Is this a new project from scratch, or are we working on an existing codebase?"
  - Based on the answer, agent selects Greenfield or Brownfield mode
  - Agent then begins the appropriate discovery Q&A for that mode (project name, problem statement, goals, etc.)
- Postconditions:
  - Mode is recorded (Greenfield or Brownfield)
  - Discovery answers captured in project brief artifact
- Error handling:
  - If required governance file missing, agent lists what is missing and stops before asking any project questions
  - If mode answer is ambiguous, agent asks one clarifying question before proceeding

Example startup transcript (Greenfield):
- Agent: "I have read the governance files. First question: are we building a new project from scratch or working on an existing codebase?"
- User: "New project from scratch."
- Agent: "Recorded: Greenfield mode. Next: what is the project name and one-sentence problem statement?"

Example startup transcript (Brownfield):
- Agent: "I have read the governance files. First question: are we building a new project from scratch or working on an existing codebase?"
- User: "Existing codebase; refactor."
- Agent: "Recorded: Brownfield mode. Next: what baseline behavior must remain unchanged and where is runtime evidence available?"

### FR-002 Stage approval behavior
- Preconditions:
  - Current stage done criteria satisfied
- Trigger:
  - User gives explicit approval
- Expected behavior:
  - Stage closes
  - Stage-completion commit required before next-stage work
- Postconditions:
  - Approval and commit recorded
- Error handling:
  - Without explicit yes, transition denied

### FR-003 Task discipline behavior
- Preconditions:
  - Task exists in task list
- Trigger:
  - Task start or task completion event
- Expected behavior:
  - Start allowed only when DoR is met
  - Done allowed only when DoD is met
- Postconditions:
  - Task status reflects real quality gate status
- Error handling:
  - Task moves or remains Blocked with missing criteria listed

### FR-004 Collaboration behavior
- Preconditions:
  - Work item has ambiguity
- Trigger:
  - Agent detects conflicting interpretations
- Expected behavior:
  - Agent consults remit-holder peer first
  - Escalate to Lefty only if unresolved
- Postconditions:
  - Clarification trail documented
- Error handling:
  - Premature escalation is treated as process violation and corrected

### FR-005 Brownfield safety behavior
- Preconditions:
  - Brownfield mode active in a future project
- Trigger:
  - Team attempts to commit to feature delivery
- Expected behavior:
  - Discovery timebox and baseline evidence completed first
  - Characterization/parity checks defined before promise
- Postconditions:
  - Feature commitments tied to confidence gate
- Error handling:
  - If confidence threshold not met: extend discovery, reduce scope, or stabilization sprint

## 5. Non-Functional Requirements Mapping

- NFR-001
  - Dimension: Process & Workflow
  - Metric: stage transition violations per project
  - Target: 0 unauthorized stage transitions
  - Validation method: review approval trail and stage records
- NFR-002
  - Dimension: Maintainability Over Time
  - Metric: master-template modifications per new project
  - Target: downward trend over successive projects
  - Validation method: compare modification counts across projects
- NFR-003
  - Dimension: Reliability & Resilience
  - Metric: missing-traceability defects at stage review
  - Target: 0 critical traceability gaps at stage closure
  - Validation method: traceability matrix audit
- NFR-004
  - Dimension: Developer Experience
  - Metric: time to start guided discovery in a new project
  - Target: immediate start with no manual policy reconstruction
  - Validation method: dry-run startup check

## 6. Data and Interface Contracts

- Input contracts:
  - UserPrompt: free text, timestamp, author
  - StageApproval: stage name, explicit yes/no, approver
  - TaskUpdate: task ID, status, links
- Output contracts:
  - PromptLogEntry in prompts.md
  - StatusSnapshot in memory.md
  - Stage artifacts (brief, spec, task list, chronicle)
- API/protocol definitions:
  - File-based markdown protocol in repository
- Versioning strategy:
  - Increment spec version on structural changes
  - Record approvals in artifact stage-approval section

Example:
- When approval input is "yes", system writes approver/date and allows transition.

## 7. Architecture and Design Decisions

- Decision: Keep governance artifacts as markdown files in repo
  - Rationale: transparent, reviewable, version-controlled
  - Alternatives considered: external tracking tools
  - Consequences: easy audit, manual discipline still required

- Decision: Enforce explicit milestone commit on stage completion
  - Rationale: frequent rollback points and cleaner history
  - Alternatives considered: idle-only commit automation
  - Consequences: slightly more commits, better recovery safety

- Decision: Start with simple tracker, add automation later
  - Rationale: reduce rollout friction and cognitive load
  - Alternatives considered: full automation in v1
  - Consequences: early manual work, faster adoption

## 8. Test Strategy (TDD-aligned)

- Unit test approach:
  - Validate stage-gate rules as isolated decisions
  - Example: transition denied without explicit approval
- Integration test approach:
  - Simulate stage progression with artifact generation and approval flow
  - Example: Discover -> Specify only after approval plus milestone commit
- Acceptance test approach:
  - Run end-to-end project bootstrap from empty project context
  - Verify required artifacts and logs are created in order
- Exploratory test focus areas:
  - Ambiguity handling
  - Cross-agent handoff failures
  - Brownfield confidence gating edge cases

## 8.1 Downstream Implementation Chronicle Expectations

- Required chronicle entries or modules:
  - Stage gate evaluator behavior
  - Task status and DoR/DoD enforcement logic
  - Escalation-routing behavior
- Implementation constraints that must be recorded:
  - Any deviations from expected stage flow
  - Any simplified assumptions introduced during coding
- Areas where alternatives are expected:
  - Tracker implementation approach (markdown-only vs lightweight automation)
- Reconstruction-critical details:
  - How approval, commit, and transition checks are linked

## 9. Traceability Matrix

- FR-001
  - Spec section: 4 (FR-001), 6
  - Planned tests: startup discovery flow test
  - Planned implementation chronicle entry: CHR-GOV-STARTUP
- FR-002
  - Spec section: 4 (FR-002), 7
  - Planned tests: stage transition gate test
  - Planned implementation chronicle entry: CHR-GOV-STAGE-GATE
- FR-003
  - Spec section: 4 (FR-003), 8
  - Planned tests: DoR/DoD enforcement tests
  - Planned implementation chronicle entry: CHR-GOV-TASK-DISCIPLINE
- FR-004
  - Spec section: 4 (FR-004)
  - Planned tests: ambiguity routing test
  - Planned implementation chronicle entry: CHR-GOV-COLLAB
- FR-005
  - Spec section: 4 (FR-005), 8
  - Planned tests: brownfield confidence gate tests
  - Planned implementation chronicle entry: CHR-GOV-BROWNFIELD-GATE
- NFR-001
  - Spec section: 5
  - Planned tests: stage-transition audit check
  - Planned implementation chronicle entry: CHR-GOV-AUDIT
- NFR-002
  - Spec section: 5
  - Planned tests: trend-measurement script/checklist
  - Planned implementation chronicle entry: CHR-GOV-MAINTAINABILITY

## 10. Stage Approval

- Approved by: Lefty
- Approval date: 2026-03-19
- Notes: Section-by-section walkthrough completed; wording adjusted for practical, example-first clarity.
