# Phase 25: Evaluator & Function Dispatch - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

AST evaluator connecting all 79 q-Kangaroo functions to qsym-core, with a variable environment for session state, text output formatting, and Maple alias resolution. The parser (Phase 24) is complete. The REPL shell (Phase 26) and LaTeX output (Phase 27) are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Maple alias table
- Claude's Discretion: case-sensitivity of aliases (recommend case-insensitive for researcher friendliness)
- Claude's Discretion: scope of alias table (recommend all Garvan names where they differ from q-Kangaroo)
- Claude's Discretion: whether to show alias notice (recommend silent — just works)
- Claude's Discretion: truncation parameter handling (recommend optional with session default, closer to Maple)

### Output formatting
- Non-series results (findlincombo vectors, polynomials, product forms) should use Maple-style formatting
- Claude's Discretion: series display term count (recommend show all computed terms up to a reasonable limit)
- Claude's Discretion: integer result display format (recommend just the number, no context label)
- Claude's Discretion: product form display (recommend product notation as primary, matching Maple's prodmake output)

### Error behavior
- Wrong argument count errors MUST show expected signature: "Error: aqprod expects 4 arguments (a, q, n, N), got 2"
- REPL MUST catch panics from qsym-core (e.g., division by zero) and continue session — never crashes
- Unknown function errors MUST suggest similar names: "Error: unknown function 'etaq2'. Did you mean: etaq, theta2?"
- Claude's Discretion: runtime error detail level (recommend user-friendly messages, not internal stack traces)

### Function argument conventions
- Variables and inline expressions are interchangeable — evaluator resolves either (e.g., both `prodmake(f, 20)` and `prodmake(etaq(1,1,20), 20)` work)
- List arguments use bracket syntax: `findlincombo([f, g, h], 20)` — requires parser extension for `[...]` list literals
- Claude's Discretion: session parameter handling (recommend implicit — user never sees session, unlike Python API)
- Claude's Discretion: hypergeometric function syntax (recommend lists for num/den parameters, matching bracket convention)

### Claude's Discretion
- Case-sensitivity of Maple aliases
- Scope and content of alias table
- Alias notification behavior
- Truncation parameter optionality
- Series display term limit
- Integer result format
- Product form display style
- Runtime error detail level
- Session parameter visibility
- Hypergeometric argument syntax

</decisions>

<specifics>
## Specific Ideas

- Parser needs `[...]` list literal support (AstNode::List) for functions like findlincombo that take series lists
- Bracket syntax matches Maple's list notation: `[f, g, h]`
- Both `prodmake(f, 20)` and `prodmake(etaq(1,1,20), 20)` must work — evaluator recursively evaluates arguments before dispatching
- Error signatures should match the function's actual parameters, not internal Rust types
- Panic catching via `std::panic::catch_unwind` in the evaluation loop

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 25-evaluator-function-dispatch*
*Context gathered: 2026-02-17*
