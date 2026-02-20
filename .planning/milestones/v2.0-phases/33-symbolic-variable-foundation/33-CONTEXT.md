# Phase 33: Symbolic Variable Foundation - Context

**Gathered:** 2026-02-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Parser and evaluator support bare symbols, q-as-parameter, and q-monomial/polynomial arguments. Users can type undefined names, pass `q` (or any symbol) as a function argument, and use q-polynomials like `q^2 + q + 1` as parameters. This is the prerequisite for all subsequent Maple-compatible function signatures in Phases 34-40.

</domain>

<decisions>
## Implementation Decisions

### Symbol fallback behavior
- Silent Maple-like: typing an undefined name returns a Symbol with no warning or annotation
- ALL undefined names become symbols (not just single-letter) -- matches Maple exactly
- No typo detection or "did you mean?" suggestions

### q variable status
- `q` is NOT pre-defined or special -- it behaves like any other undefined name (becomes a symbol on first use), matching Maple's behavior when importing the q-series package
- `q` is NOT protected from reassignment -- `q := 5` is allowed (Maple-like, user's responsibility)
- Any symbol can be used as the base variable -- `etaq(t, 1, 20)` works and produces a series in `t`
- Series internally tracks which symbol was used as its variable and displays accordingly -- `etaq(t, 1, 20)` displays terms in `t`, not `q`

### Monomial and polynomial forms
- Full q-polynomials accepted as function arguments -- `q^2`, `q^(-1)`, `2*q^3`, `q^2 + q + 1` all work
- Standalone polynomial arithmetic works at the REPL -- `(q^2 + 1) * (q + 1)` evaluates and returns `q^3 + q^2 + q + 1`
- Display matches Maple: polynomials (finite terms) display as exact polynomials without truncation; infinite series display with `O(q^n)` notation

### Variable clearing
- Maple unassign syntax: `x := 'x'` turns a defined variable back into a bare symbol
- `restart` command clears all user-defined variables at once (Maple-style)
- Variable listing command available (like Maple's `anames()`) to show all currently defined variables and values

### Claude's Discretion
- Symbol arithmetic outside function calls: how symbols behave in expressions like `f + 1` or `a * b` (minimum needed for phase success criteria)
- Symbol display formatting in LaTeX mode
- Internal representation of q-polynomials passed to functions (evaluate to FormalPowerSeries vs keep symbolic)
- What exactly `restart` clears (user vars only vs also `ans` and output history)

</decisions>

<specifics>
## Specific Ideas

- "Should behave exactly as q would when importing the q-series package" -- the guiding principle for `q` handling
- "Match Maple exactly" for polynomial display -- polynomials are polynomials, series are series with O(...)
- The system should feel like Maple to a researcher copy-pasting from Maple worksheets

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 33-symbolic-variable-foundation*
*Context gathered: 2026-02-18*
