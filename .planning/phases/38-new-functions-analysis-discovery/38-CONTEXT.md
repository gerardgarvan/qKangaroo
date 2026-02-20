---
phase: 38-new-functions-analysis-discovery
created: 2026-02-19
status: confirmed
---

# Phase 38 Context: New Functions - Analysis & Discovery

## Scope

Implement 4 analysis/discovery functions matching Garvan's Maple qseries package:
- `checkmult` -- test if coefficients are multiplicative
- `checkprod` -- test if series is a "nice" formal product
- `lqdegree0` -- lowest q-degree of a monomial term
- `findprod` -- exhaustive search for product identities

**Deferred:** `zqfactor(F, z, q, N)` moved to a future phase (requires bivariate series infrastructure that doesn't exist yet).

## Decisions

### 1. Signature Corrections (Verified Against Garvan Source)

The original REQUIREMENTS.md had wrong signatures for all 5 functions. Corrected from Garvan's actual Maple source (qseries v1.3j, wprog-qseries-06-13-2020.txt):

| Function | Wrong (old) | Correct (Garvan) |
|---|---|---|
| checkmult | `checkmult(f, q, T)` | `checkmult(QS, T)` or `checkmult(QS, T, 'yes')` |
| checkprod | `checkprod(f, q, T)` | `checkprod(f, M, Q)` -- M is max-exponent threshold |
| lqdegree0 | `lqdegree0(f, q)` | `lqdegree0(qexp)` -- 1 arg only |
| zqfactor | `zqfactor(f, z, q)` | `zqfactor(F, z, q, N [, buglim])` -- 4-5 args |
| findprod | `findprod(L, q, maxcoeff, maxexp)` | `findprod(FL, T, M, Q)` -- no q arg |

### 2. checkmult Behavior

- **Signature:** `checkmult(f, T)` with optional 3rd arg `'yes'` (string, using single-quote support from Phase 33)
- **Algorithm:** Check f(mn) = f(m)*f(n) for all coprime pairs m,n with 2 <= m,n <= T/2 and mn <= T
- **2-arg output:** Print "MULTIPLICATIVE" or "NOT MULTIPLICATIVE" with first failing (m,n) pair. Return 1 or 0.
- **3-arg output (with 'yes'):** Print ALL failing (m,n) pairs. Return 1 or 0.
- **No explicit q arg** -- series values are already q-series in our CLI

### 3. checkprod Behavior

- **Signature:** `checkprod(f, M, Q)` -- f is series, M is max absolute exponent threshold, Q is truncation order
- **Algorithm:** Runs prodmake internally. Normalizes series (strip q^a factor, divide by leading coefficient). Checks if all product exponents have |A[n]| < M.
- **Return values (silent, no printing):**
  - `[a, 1]` -- nice product (all exponents < M)
  - `[a, max_exp]` -- not nice (max_exp >= M)
  - `[[a, c0], -1]` -- leading coefficient not integer-divisible
- **Return type:** Value::List of Value::Integer/Value::Rational

### 4. lqdegree0

- **Signature:** `lqdegree0(f)` -- 1 arg, FPS values only
- **Returns:** Minimum key in the FPS BTreeMap (lowest non-zero coefficient degree)
- **Rationale:** In Garvan's Maple, lqdegree0 handles symbolic Maple expressions (JAC*q^(1/2)). In our CLI, FPS already stores coefficients with explicit powers. JacobiProduct users can convert with jac2series first.
- **Relationship to lqdegree:** In our CLI, lqdegree0 and lqdegree are nearly identical for FPS inputs (both return the lowest degree). The distinction exists primarily for Maple compatibility. lqdegree already exists; lqdegree0 is added as a Garvan-compatible alias/equivalent.

### 5. findprod Behavior

- **Signature:** `findprod(FL, T, M, Q)` -- FL is list of series, T is max |coefficient|, M is max product exponent, Q is truncation order
- **Algorithm:** Exhaustive search over all integer coefficient vectors with |c_i| <= T. For each primitive vector (gcd = 1), form linear combination, call checkprod(combo, M, Q). Collect nice products.
- **Return (silent, no printing):** List of [valuation, coefficient_vector] pairs
- **Performance:** No artificial limits. Search space is (2T+1)^nops(FL) vectors. User is responsible for choosing reasonable T.
- **Dependency:** Calls checkprod internally

### 6. zqfactor Deferral

- **Decision:** Move NEW-08 (zqfactor) to a future phase
- **Reason:** Requires bivariate series infrastructure (coefficients as polynomials in z, symbolic z in product functions, bivariate display). None of this exists in the current codebase.
- **Approach:** Create a dedicated phase for bivariate support when needed (v2.1 or later)

### 7. Output Patterns

- **checkmult:** Prints messages (MULTIPLICATIVE / NOT MULTIPLICATIVE + failure details). Returns integer 1 or 0.
- **checkprod:** Silent return only. Returns Value::List matching Garvan's [a, code] format.
- **lqdegree0:** Silent return only. Returns Value::Integer.
- **findprod:** Silent return only. Returns Value::List of [valuation, coefficient_vector] sub-lists.

## Deferred Ideas

- Bivariate (z,q)-series type and zqfactor implementation (future milestone)
- lqdegree0 support for JacobiProduct inputs (convert via jac2series if needed)

## Implementation Notes

- checkprod depends on existing prodmake (already in qsym-core, exposed to CLI in Phase 35)
- findprod depends on checkprod (build checkprod first)
- checkmult and lqdegree0 are independent
- All functions use existing FPS/Value infrastructure -- no new types needed
