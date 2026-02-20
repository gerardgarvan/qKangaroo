# Phase 37: New Functions - Theta & Jacobi - Context

**Gathered:** 2026-02-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement four new functions: `theta(z, q, T)`, `jac2prod(JP, q, T)`, `jac2series(JP, q, T)`, and `qs2jaccombo(f, q, T)`. These enable workflows that convert between theta series, Jacobi product expressions, and q-series representations. This phase introduces a new `JacobiProduct` value type and the `JAC(a,b)` constructor function.

</domain>

<decisions>
## Implementation Decisions

### Jacobi product input format
- `JAC(a,b)` is a function call that returns a `JacobiProduct` value representing (q^a; q^b)_inf
- JacobiProduct is a standalone value type: assignable to variables, printable, combinable with `*`, `/`, and `^`
- Full algebra supported: `JAC(1,5) * JAC(2,5)`, `JAC(1,5) / JAC(2,5)`, `JAC(1,5)^3` all work
- Integer exponents (positive and negative) supported for power notation
- JP expressions are products/quotients of JAC factors with integer exponents
- `jac2prod` and `jac2series` strictly require a JacobiProduct value -- passing a non-JP value errors with "expected Jacobi product expression (use JAC(a,b))"

### theta(z, q, T) behavior
- `theta(z, q, T)` computes sum(z^i * q^(i^2), i=-T..T)
- If z is a numeric value (integer, rational): substitute and return univariate q-series
- If z is a q-monomial (e.g., q^2): auto-substitute z=q^k and return univariate q-series (e.g., sum(q^(2i + i^2)))
- If z is a bare symbol (unassigned): print a warning message that z must be numeric or a q-monomial, do NOT error -- just warn
- General form only -- classical theta2/3/4 already exist as separate functions, no variants needed
- No artificial T limit -- user chose T, user gets the result

### Conversion output formats
- All three conversion functions both **print** human-readable output AND **return** the value (consistent with find* functions from Phase 36)
- `jac2prod(JP, q, T)`: print product notation string like `(1-q)(1-q^2)(1+q^3)...`, return the FPS value
- `jac2series(JP, q, T)`: display behavior at Claude's discretion (standard series display likely best)
- `qs2jaccombo(f, q, T)`: print JAC expression formula like `2*JAC(1,5)*JAC(2,5) + 3*JAC(3,5)`

### Error handling
- `JAC(a,b)` validates: b must be a positive integer, a must be an integer. Error with clear message on invalid args
- `qs2jaccombo` when no decomposition found: print "No Jacobi product decomposition found", return input series unchanged
- `jac2prod`/`jac2series` require strict JacobiProduct input -- clear error for wrong type

### Claude's Discretion
- Exact `jac2series` display format (standard series display vs. something specialized)
- Internal representation of JacobiProduct (Vec of (a, b, exponent) factors, or similar)
- Algorithm choice for qs2jaccombo decomposition
- How JacobiProduct values display when printed standalone (e.g., "JAC(1,5)*JAC(2,5)^(-1)")

</decisions>

<specifics>
## Specific Ideas

- JAC(a,b) notation matches Garvan's Maple JAC(a,b) exactly -- researchers see familiar syntax
- Product notation for jac2prod should look like explicit factors: (1-q)(1-q^2)... not abstract (a;q)_inf notation
- The print-and-return pattern matches Phase 36's find* functions, giving consistent UX across the CLI

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 37-new-functions-theta-jacobi*
*Context gathered: 2026-02-19*
