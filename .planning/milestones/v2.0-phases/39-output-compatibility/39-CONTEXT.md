---
phase: 39-output-compatibility
created: 2026-02-19
status: confirmed
---

# Phase 39: Output & Compatibility - Context

**Gathered:** 2026-02-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Series display matches Maple conventions (descending polynomial ordering) and all existing v1.x calling conventions still work after the Phase 33-38 signature changes. No new functions -- this is about output formatting and backward compatibility verification.

</domain>

<decisions>
## Implementation Decisions

### Polynomial Display Ordering
- **Default ordering: descending (Maple-style)** -- highest power first for all series output
- Both plain text and LaTeX output use descending ordering
- No user toggle -- one default, always descending
- Applies to both infinite series (with O(q^T)) and exact polynomials
- POLYNOMIAL_ORDER sentinel behavior for truncation markers: Claude's discretion

### Old Signature Handling
- **No deprecation warnings** -- old v1.x signatures work silently, both forms are first-class forever
- Replaced signatures (e.g., old 3-arg findprod): Claude decides whether to give helpful error or standard arg-count error
- **Verify ALL functions** that changed signatures: etaq, aqprod, jacprod, tripleprod, quinprod, winquist, qbin, sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor, and all find* functions
- Old-style tests that no longer match: update to new Garvan-compatible signatures

### Test Expectation Updates
- **Update all assertions** that check output ordering -- fix every test to match new descending format
- **Fix proactively** -- review and update integration tests checking output ordering BEFORE making the display change
- **Update roadmap success criteria** to current test counts (281 core + 549 CLI), not outdated v1.6 counts
- **Add dedicated backward-compat test group** -- new "backward_compat" section in cli_integration.rs testing all old-style function calls work correctly

### Claude's Discretion
- POLYNOMIAL_ORDER sentinel behavior for truncation marker display
- Whether replaced signatures get helpful migration messages or standard arg-count errors
- Test organization details within the backward_compat group

</decisions>

<specifics>
## Specific Ideas

No specific requirements -- standard Maple polynomial ordering convention (descending powers) is well-defined.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 39-output-compatibility*
*Context gathered: 2026-02-19*
