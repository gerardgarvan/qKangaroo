# Phase 24: Parser & AST - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Maple-style expression parser that converts user input into an internal AST representation. Handles function calls, variable assignment, arithmetic, literals, and keywords. The evaluator (Phase 25) and REPL shell (Phase 26) are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Syntax fidelity
- Use `:=` for assignment (Maple style), not `=`
- Maple-style semicolons: `;` terminates and displays result, `:` terminates and suppresses output
- `%` refers to the last computed result (Maple convention)
- Function names: support both existing q-Kangaroo names (aqprod, etaq, partition_count) AND Maple aliases (ETAR, JACPROD, etc.)
- Exponentiation with `^` operator (Maple convention)

### The q variable
- `q` is a built-in symbol, always available as the series indeterminate — no need to define it
- Other symbolic variables (a, z, x, etc.) are allowed in function arguments, matching Maple flexibility
- Undefined names are errors — variables must be assigned with `:=` before use (except `q` and `infinity` which are built-in)

### Rational literals and operators
- Claude's Discretion: whether `3/4` parses as rational literal or division — pick what's least surprising
- Claude's Discretion: whether `/` is supported as series division operator
- `^` is supported for exponentiation (user decision)

### Expression chaining
- Multiple statements on one line separated by `;` or `:` — Maple style
- Bare Enter (no `;` or `:`) auto-evaluates and displays — forgetting `;` is not an error
- Each `;`-terminated statement prints its result; `:` suppresses — multiple outputs per line possible
- Assignments (`f := expr;`) print the assigned value — Maple behavior

### Claude's Discretion
- Nature of `q` (reserved keyword vs pre-defined variable)
- `3/4` rational literal vs division semantics
- Whether to support `/` for series division
- Operator precedence details
- Parse error message format and detail level

</decisions>

<specifics>
## Specific Ideas

- The goal is a Maple-faithful feel: researchers coming from Garvan's Maple packages should feel at home immediately
- `f := aqprod(q,q,infinity,20); g := etaq(2,1,20); f * g;` should work as a single line
- `f := etaq(1,1,20):` assigns silently; `f;` displays the series
- Maple aliases mean users can type either `etaq(1,1,20)` or `ETAR(1,1,20)` and get the same result

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 24-parser-ast*
*Context gathered: 2026-02-17*
