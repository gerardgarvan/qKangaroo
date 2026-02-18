# Phase 32: PDF Reference Manual - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Produce a comprehensive, professionally typeset PDF reference manual covering all 81 q-Kangaroo functions, CLI usage, worked examples, and a Maple migration quick-reference. The PDF is included in GitHub release archives alongside the binary. This phase writes documentation only — no code changes except updating `--help` to mention the manual.

</domain>

<decisions>
## Implementation Decisions

### Document structure & flow
- Hybrid organization: introductory chapters followed by an alphabetical function reference
- Intro chapters: Quick Start tutorial (2-3 pages: install, first expression, first script), CLI usage summary (flags/modes, brief exit code mention), expression language reference (variables, operators, types)
- Function reference section: Claude's discretion on whether to group by domain or flat A-Z — pick whichever makes 81 functions most navigable
- Comprehensive back-of-book index: function names, math concepts, and Maple function names cross-referenced
- Target length: 100+ pages (comprehensive treatment)
- CLI chapter: summary-level coverage of exit codes and error messages (not a sysadmin reference)

### Function entry depth
- Full treatment per function: signature, mathematical definition, parameter descriptions, 2-3 examples, edge cases, related functions
- Mathematical notation: formal product/sum notation for core q-series functions (qpoch, etaq, jacprod, etc.); informal prose descriptions for utility/helper functions
- Maple equivalents: NOT in individual function entries — all Maple mappings go in the migration quick-reference table only
- Examples: REPL transcript style showing `q>` input and actual output
- Edge cases: document parameter constraints, invalid input behavior, and known limitations

### Typesetting tool
- Typst (not LaTeX) — modern, fast, single binary, Rust-based
- CI builds PDF on release via GitHub Actions (install Typst, compile from source)
- Page layout and file organization: Claude's discretion

### Worked examples & migration
- Both identity verification examples AND research workflow examples
- Number of examples: Claude's discretion based on function families and document balance
- Cite source papers for identities (e.g., "This proves Theorem 2.1 from Andrews 1986")
- Maple migration: side-by-side mapping table only (no prose translation guide)

### Claude's Discretion
- Function reference grouping (by domain vs flat A-Z)
- Page layout (single column vs mixed)
- Typst file organization (single file vs split by chapter)
- Number and selection of worked examples
- Overall tone and style (blend of Maple help pages and mathematical reference)
- Font choices and visual design

</decisions>

<specifics>
## Specific Ideas

- REPL transcript style for all code examples: show `q>` prompt, input, and output
- Back-of-book index should cross-reference Maple function names so users can look up "qpochhammer" and find "qpoch"
- Worked examples should cover both proving known identities (Jacobi triple product, Rogers-Ramanujan) and exploratory research workflows
- Academic citations for identity examples add credibility with the math researcher audience

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 32-pdf-reference-manual*
*Context gathered: 2026-02-18*
