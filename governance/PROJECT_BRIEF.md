# Project Brief

Purpose: capture Layer 1 command intent for the current project and drive Stage 1 discovery questions.

Layer metadata: Layer 1 of the three-layer documentation stack (Commander's Intent -> Behavioral Specification -> Implementation Chronicle).
Expected downstream links: FORMAL_SPEC.md (Layer 2) and IMPLEMENTATION_CHRONICLE.md (Layer 3).

## 1. Project Overview

- Project name: Competitor Spy
- Project mode: Greenfield
- Primary implementation language: Rust
- Secondary implementation language(s): Bash (Linux/Git Bash launcher), PowerShell (Windows launcher)
- Language decision status: Fixed
- Current delivery phase: V3 discovery (builds on V2 studio list baseline)
- Problem statement: V2 finds and ranks nearby competitors, but website-level business intelligence is still shallow. V3 must enrich each discovered studio with actionable website-derived details for pilates market analysis.
- Desired business/domain outcome: Produce richer competitor intelligence from studio websites so business decisions can consider pricing, offering depth, and class structure, not only discovery/ranking metadata.

In-scope goals (V3):
- Reuse V2 competitor list as source set (26 studios in current target run).
- Scrape each studio website for enrichment data where publicly available.
- Extract and report, if present:
  - pricing/pricing models (memberships, intro offers, class prices)
  - lesson types (reformer, mat, fusion, etc.)
  - schedule/class availability
  - customer testimonials/reviews shown on-site
  - class descriptions and indicated level/difficulty
- Keep existing V2 ordering rule (distance remains the only ranking factor).
- Keep anti-ban pacing behavior and human-like request cadence.
- Generate enriched terminal and PDF reports with explicit missing-field markers when data cannot be extracted.

Out-of-scope items (V3):
- Reordering competitors by price or class variety.
- Paid third-party data providers.
- Auth-gated or private content scraping.
- Multi-user support, scheduling, or background monitoring.
- GUI/web app delivery in V3.

## 1.1 Mode-Specific Direction

- Greenfield direction: preserve existing Rust-first modular architecture and add a website-enrichment adapter pipeline.
- V3 architectural intent: continue using V2 result set for discovery, then run enrichment stage per studio URL.
- Extraction strategy selected during Stage 1: lightweight Rust HTTP + HTML parsing first (reqwest + scraper pattern), with graceful partial enrichment when site structure differs.

## 1.2 Quality Module Declarations

- Data Quality module active? Yes
  - Trigger: partial extraction, missing-field signaling, and cross-site schema normalization.
- Compliance and Auditability module active? Yes
  - Applicable regulations: GDPR-aligned handling; public data only; no private/auth-protected data scraping.
- Interactive CLI diagnostics required? Yes
  - Trigger: CLI output validation and manual result review in exploratory sessions.
  - Capture method: screen-state capture + application-state capture helpers.
  - Storage location: docs/evidence/sessions/ with naming session_YYYYMMDD_HHMMSS_<label>.{log,json}
- Security and production-readiness loop required? Yes
  - Trigger: network scraping, external site interaction, and rate-limiting/ban-risk controls.
- Layered architecture constraint active? Yes (Q3-ARCH-01)
  - Interface -> API -> CLI layering remains enforced.

## 2. Stakeholders and Users

- Sponsor: Lefty
- Product owner: Lefty
- Primary user groups: solo operators evaluating pilates-market competition in target geography
- Secondary user groups: investors and advisors consuming report outputs

## 3. Functional Requirements

1. FR-001: Use V2 competitor outputs as the canonical input set for V3 enrichment.
2. FR-002: For each studio with a website URL, fetch and parse website content for target enrichment fields.
3. FR-003: Extract pricing/pricing-model information where available and label source context.
4. FR-004: Extract lesson/service types where available.
5. FR-005: Extract schedule/class-availability information where available.
6. FR-006: Extract testimonials/review snippets published on the studio website where available.
7. FR-007: Extract class descriptions and level/difficulty indicators where available.
8. FR-008: Preserve V2 distance ordering and append V3 enrichment without rank reordering.
9. FR-009: If a field cannot be extracted, mark it explicitly as unavailable; do not fail the whole run.
10. FR-010: Continue anti-ban pacing with randomized delays and human-like crawl cadence.
11. FR-011: Produce terminal and PDF outputs with new enrichment sections.

## 4. Non-Functional Expectations

- Performance: acceptable completion for ~26 studio websites in one run with intentional pacing.
- Reliability/availability: partial enrichment is required; failed pages/fields do not abort full report generation.
- Security/privacy: public pages only, no credentialed/private access, secrets redacted in logs.
- Scalability: single-run, single-user execution in V3 scope.
- Observability: structured per-site extraction status and per-field availability logging.
- Maintainability: extraction rules should be modular and extendable by website pattern/source adapter.
- Compliance/regulatory: GDPR-aligned public-data usage and auditability retained.

## 4.1 Determinism and Rebuild Constraints

- Deterministic constants: existing radius options and distance-first ordering policy remain unchanged.
- RNG contract: randomness allowed for pacing jitter only.
- Tie/ordering policy: unchanged from V2 (distance primary).
- I/O contract: enriched report keeps UTC timestamp naming and UTF-8 encoding.
- Runtime environments: Linux x86_64 and Windows 11 x86_64 required.

## 4.2 Acceptance Scenarios (User-Visible)

1. Scenario ID: AS-V3-001
   - Given: V2 list includes studio URLs
   - When: V3 enrichment runs
   - Then: report includes new sections for pricing, lesson types, schedules, testimonials, and class descriptions when found.

2. Scenario ID: AS-V3-002
   - Given: website content is partial or structure differs
   - When: extraction cannot resolve all target fields
   - Then: fields are marked unavailable, run continues, and report remains complete for available data.

3. Scenario ID: AS-V3-003
   - Given: run target is pilates, Neulengbach, Austria, 50 km
   - When: V3 enrichment executes on the resulting studio set
   - Then: ranking remains distance-based and enriched content is appended without reordering.

## 5. Domain Constraints and Assumptions

- Constraint 1: scrape only publicly available website content.
- Constraint 2: no private/login/paywalled scraping.
- Constraint 3: V3 enrichment depends on V2-provided URLs and should not replace V2 discovery in this phase.
- Constraint 4: extracted competitor data remains report-scoped for current run outputs.
- Assumption 1: most target studio sites are static/server-rendered or lightly scripted and suitable for HTTP+HTML parsing.
- Assumption 2: extraction quality will vary by site structure and language/content conventions.

## 6. Interfaces and Dependencies

- Upstream: website URLs derived from V2 competitor results.
- Downstream: terminal output and PDF report.
- External services: direct HTTP access to studio websites.
- Data stores: existing credential/audit pipeline remains; no new persistent competitor datastore introduced in Stage 1.

## 7. Acceptance Criteria

1. AC-001: Enrichment run adds at least one new V3 field for studios where data is present.
2. AC-002: Missing/failed extraction paths are explicitly labeled and do not terminate full report generation.
3. AC-003: Output includes dedicated enrichment sections in terminal and PDF reports.
4. AC-004: Distance ranking remains unchanged from V2 baseline.
5. AC-005: Pacing and anti-ban behavior remains active during website requests.

## 8. Risks and Unknowns

- Risk 1: site structure drift can break extraction patterns.
- Risk 2: anti-bot protections may reduce available content on some websites.
- Risk 3: multilingual content and inconsistent pricing terminology may reduce extraction precision.
- Unknown U-V3-001: exact extraction coverage threshold required for acceptable V3 quality gate.
- Unknown U-V3-002: whether any target sites require JS rendering fallback in later iteration.

## 8.1 Brownfield Legacy Uncertainty Handling

- Not applicable. Mode is Greenfield.

## 8.2 Approval Authority and Delegation

- Delegation mode: Team lead for all stages
- Delegated approver role: Team Lead Agent
- Delegation start stage: Stage 2
- Delegation end condition: through Stage 6 unless explicitly revoked by Lefty
- Escalate-to-owner conditions: scope change, compliance risk change, or legal/operational dependency changes

## 9. Stage 1 Approval

- Status: **APPROVED — 2026-03-24**
- Approved by: Lefty (user/owner)
- Ready-for-approval summary: V3 discovery scope captured for website enrichment over V2 studio list, Rust-first stack confirmed, partial-enrichment failure behavior confirmed, report-section expansion confirmed, and ranking rule preserved.
- Delegation decision: All decision-making and approval authority for Stages 2–6 is formally delegated to the Team Lead Agent, acting in accordance with the governance structure. Team Lead may coordinate all participating agents and uphold expected protocols. Escalation to Lefty required only for scope change, compliance risk change, or new legal/operational dependencies.
