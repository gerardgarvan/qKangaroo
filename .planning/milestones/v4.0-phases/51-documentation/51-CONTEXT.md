# Phase 51: Documentation - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

All v4.0 features documented in the help system, tab completion, and PDF manual with worked examples from qmaple.pdf. No new functionality — documentation only for features already implemented in Phases 47-50.

</domain>

<decisions>
## Implementation Decisions

### Help text depth
- Full help entries with signature, description, 2-3 examples, and edge case notes
- Examples taken directly from qmaple.pdf so users can cross-reference
- No Maple references in help text — entries should be self-contained
- Update existing help entries for functions that got new signatures in v4.0 (theta3 2-arg, qfactor 2-arg, aqprod 3-arg)
- Include help for both min() and max()

### PDF manual structure
- Organize v4.0 chapter by feature type: Language Features, Bug Fixes, New Functions
- Include a full walkthrough section reproducing ALL executable qmaple.pdf examples
- Include a "Not Yet Supported" subsection listing examples that require deferred features (while loops, zqfactor, etc.)
- Examples show commands + expected output (REPL transcript style)
- Match qmaple.pdf presentation style — same sequence of commands in same order
- Reference qmaple.pdf by both section number and page: "Section 3.2 (p.12)"

### Tab completion scope
- Add function names only: jac2series, radsimp, quinprod, subs, min, max
- No keyword completions (prodid/seriesid are just string arguments)

### Claude's Discretion
- Which features get help entries (all new functions certainly, language syntax features at Claude's judgment)
- Whether arrow/ditto get special completion behavior
- How to organize multi-feature examples (grouped in walkthrough vs duplicated per feature)

</decisions>

<specifics>
## Specific Ideas

- Walkthrough should cover ALL executable examples from qmaple.pdf, not just highlights
- Examples presented as REPL transcripts: command followed by expected output
- Users should be able to use the walkthrough to verify their installation produces identical results

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 51-documentation*
*Context gathered: 2026-02-21*
