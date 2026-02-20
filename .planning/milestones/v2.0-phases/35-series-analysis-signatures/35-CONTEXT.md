# Phase 35: Series Analysis Signatures - Context

**Gathered:** 2026-02-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Make series analysis functions (sift, prodmake, etamake, jacprodmake, mprodmake, qetamake, qfactor) accept Garvan's exact calling conventions with explicit q parameter and Maple argument order. This phase changes signatures, updates tests, and updates help text. New functions and output display overhauls belong in other phases.

</domain>

<decisions>
## Implementation Decisions

### Backward Compatibility Strategy
- **No backward compat** -- old signatures (without explicit q) are removed, not kept as aliases
- Functions require the new Maple-style argument lists; old arity forms produce a standard "wrong number of arguments" error with usage hint
- All existing tests for these functions must be rewritten to use the new Maple-style signatures in this phase
- Function names: Claude's discretion to check Garvan's exact Maple names and adjust if any differ from current q-Kangaroo names

### sift Behavior
- `sift(s, q, n, k, T)` -- strict 5-arg form, T is always required (even for polynomial inputs)
- Error on invalid residue: k must satisfy 0 <= k < n, otherwise return error
- Truncation semantics for T: Claude's discretion -- check what Garvan's Maple does and match it
- Variable validation (whether to verify series is in terms of q): Claude's discretion

### Make-Function Output
- prodmake/etamake output should match Maple's display notation style (not just keep current format)
- jacprodmake 3-arg vs 4-arg (P parameter) output style: Claude's discretion -- research what P does in Garvan's code
- Failed decomposition handling: Claude's discretion -- match Garvan's behavior
- qfactor output format (product display vs list): Claude's discretion -- match Garvan's output

### Error Messages
- Wrong argument types should name the parameter position: "Argument 1 (f): expected series, got integer"
- Specific q-parameter errors: Claude's discretion based on existing error patterns
- help() text for all 7 functions must be updated in this phase to show new Maple-style signatures
- Tab completion must be updated in this phase for any renamed function names

### Claude's Discretion
- Exact Garvan function names (check and rename if needed)
- sift truncation semantics (match Garvan)
- sift variable validation approach
- jacprodmake P parameter behavior and output
- Failed decomposition behavior for make-functions
- qfactor output format (match Garvan)
- q-parameter type error specificity

</decisions>

<specifics>
## Specific Ideas

- The guiding principle is "match Garvan exactly" -- when in doubt, research what Garvan's Maple code does and replicate that behavior
- Error messages should be positional: "Argument N (name): expected X, got Y"
- Help text stays in sync with actual signatures -- no stale documentation

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 35-series-analysis-signatures*
*Context gathered: 2026-02-19*
