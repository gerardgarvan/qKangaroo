# Phase 36: Relation Discovery Signatures - Context

**Gathered:** 2026-02-19
**Status:** Ready for planning

<domain>
## Phase Boundary

All `find*` relation-discovery functions accept Garvan's exact Maple argument lists, including symbolic label lists (SL). Output uses those labels in printed results. Covers: findlincombo, findhomcombo, findnonhomcombo, findlincombomodp, findhomcombomodp, findhom, findnonhom, findhommodp, findmaxind, findpoly, findcong (3 overloads).

Requirements: SIG-15 through SIG-25, OUT-01, OUT-02.

</domain>

<decisions>
## Implementation Decisions

### Symbolic Label Lists (SL)
- SL parameters are passed as bare symbols using Value::Symbol from Phase 33 (e.g., `[F1, F2, F3]`)
- SL labels must be unique -- error on duplicate labels to prevent confusing output
- SL length validation and auto-label generation for functions without SL: Claude's discretion

### Relation Output Format
- findlincombo and similar functions print formatted strings: `12*F1 + 13*F2` -- matches Maple's display-oriented output
- Modular arithmetic display (mod p annotation vs plain integers): Claude's discretion, match Garvan
- Homogeneous combo polynomial display format: Claude's discretion, match Garvan
- Multiple solutions handling (first vs all): Claude's discretion, match Garvan's behavior

### findcong Output Format
- Congruence results displayed as list `[B, A, R]` triples -- matches Garvan exactly
- When multiple congruences found, print each `[B, A, R]` on its own line
- QS parameter should match Garvan's Maple QS format (needs research)
- Overloaded forms (2/3/4-arg dispatch vs optional defaults): Claude's discretion

### No-Solution Behavior
- When no combination/relation found: print a message (e.g., "no combination found") and return successfully (exit 0) -- no result is a valid outcome, not an error
- findpoly follows the same pattern: print "no polynomial relation found" message on failure
- For modp variants: validate that p is prime at dispatch time, error on non-prime
- Short list validation for degree-n homogeneous: Claude's discretion

### Claude's Discretion
- SL length validation behavior (strict match vs lenient truncate/pad)
- Auto-generated labels for functions without SL (L1, L2... vs numeric indices)
- Modular arithmetic display conventions (match Garvan)
- Homogeneous combo polynomial display (match Garvan)
- Multiple solution handling (match Garvan)
- findcong overload strategy (arg-count dispatch vs optional defaults)
- Short-list-for-degree validation approach

</decisions>

<specifics>
## Specific Ideas

- Output should match Garvan's Maple display as closely as possible -- researchers comparing results side-by-side
- The SL label mechanism is central to usability: `findlincombo(f, [e1,e2,e3], [F1,F2,F3], q, 0)` should feel identical to the Maple call
- findcong is the most complex function in this family with 3 overloaded forms -- research Garvan's exact QS format before implementing

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 36-relation-discovery-signatures*
*Context gathered: 2026-02-19*
