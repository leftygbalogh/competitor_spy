# ADR-004: PDF Generation via printpdf (Pure Rust, No System Binary)

**Date:** 2026-03-21
**Status:** Accepted
**Spec reference:** FORMAL_SPEC.md §7, Decision 4

---

## Context

Competitor Spy must produce a PDF report on every run. Rust PDF generation options include wkhtmltopdf (system binary wrapper), headless Chrome (via external process), and pure-Rust crates such as printpdf.

## Decision

Use `printpdf 0.7` for PDF generation. Layout is implemented manually (text pages, built-in Helvetica font).

## Rationale

- **No system dependency.** printpdf compiles to native Rust with no external binary requirement. This means the tool can be distributed as a single binary without requiring the user to install wkhtmltopdf, Chromium, or a system PDF library.
- **Cross-platform.** Works identically on Windows and Linux without OS-specific setup.
- **Auditable size.** The report has a fixed structure (header + table + footer); full layout flexibility of a browser-based renderer is not needed.

## Consequences

- Table layout is implemented manually using printpdf's coordinate system. This required calibrating column widths and row spacing in code (documented in IMPLEMENTATION_CHRONICLE.md CHR-CSPY-015).
- Fonts are limited to printpdf's built-in Helvetica and Helvetica-Bold (PDF Type 1). Sufficient for the v1 report; custom fonts would require embedding TTF files.
- PDF failure (e.g. file permission error) is a warning only; the run still exits 0 and terminal output is produced. This is the correct trade-off: the user still gets their results.

## Alternatives Rejected

- **wkhtmltopdf** — requires a system binary; rejected (portability)
- **headless Chrome** — requires Chrome to be installed; overkill for a fixed-format report; rejected
