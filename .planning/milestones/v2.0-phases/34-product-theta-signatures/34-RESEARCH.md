# Phase 34: Product & Theta Signatures - Research

**Researched:** 2026-02-19
**Domain:** Maple-compatible function signature dispatch for q-series product/theta functions
**Confidence:** HIGH

## Summary

Phase 34 adds Garvan's exact Maple calling conventions for 7 product/theta functions plus the `numbpart` alias. The Garvan qseries Maple package source code (version 1.2, wmprog64.txt) has been obtained and analyzed directly, providing definitive function signatures. Phase 33 already implemented the Symbol/monomial dispatch infrastructure (`Value::Symbol`, `extract_symbol_id`, `extract_monomial_from_arg`, `POLYNOMIAL_ORDER`), and demonstrated the coexistence pattern with `etaq(q, 1, 20)` and `aqprod(q^2, q, 5)`. This phase extends that pattern to all 7 product functions and upgrades `numbpart` from alias to primary name.

The critical finding is that Garvan's signatures differ from our legacy ones in specific, well-documented ways. Each function needs its own disambiguation strategy (first-arg type detection), which Phase 33's infrastructure already supports. The `qbin` signature change is the most complex because Garvan puts `q` first: `qbin(q, m, n)` vs our legacy `qbin(n, k, order)`.

**Primary recommendation:** Implement each function's Maple-style dispatch as an if/else branch above the legacy path (same pattern as Phase 33's `aqprod` and `etaq`), using first-arg type detection. Update help text to show Maple signatures only. Add multi-delta `etaq(q, [1,2,3], 20)` as syntactic sugar that multiplies individual `etaq` calls.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Signature coexistence:**
- Silent, both work: old and new signatures coexist with no deprecation warnings
- Help system shows Maple-style signatures ONLY -- old signatures are undocumented but still functional
- Error messages reference Maple signatures ONLY -- guide users toward Garvan's conventions
- Disambiguation strategy: Claude's discretion per function (cleanest approach for each)

**etaq multi-delta:**
- Full Garvan support: both `etaq(q, 3, 20)` (single delta) and `etaq(q, [1,2,3], 20)` (multi-delta list) are implemented
- Validation matches Maple behavior -- whatever Maple does for invalid lists, we do the same
- All product functions (tripleprod, quinprod, winquist) match Garvan's exact signatures -- research each function's actual Maple signature and replicate

**numbpart naming:**
- `numbpart` is the primary name (Maple convention); `partition_count` becomes alias
- `help(partition_count)` redirects to `help(numbpart)` -- one source of truth
- `numbpart` matches full Maple signature -- research what Maple's numbpart actually accepts (overloaded forms)

**Output exactness:**
- Match Garvan's coefficient ordering exactly (ascending powers as Garvan produces)
- Finite products always expand to polynomials (not product notation)
- O(q^N) notation matches Garvan's exact format -- research and replicate character for character
- Coefficient display matches Garvan -- research whether Garvan uses `2*q^3` or `2q^3` and replicate
- Test against Garvan's actual Maple output (captured test vectors) for coefficient-by-coefficient verification

### Claude's Discretion
- Disambiguation strategy per function: first-arg type, arg count, or other approach as cleanest for each function
- numbpart primary vs equal peers in tab completion ordering
- Exact handling of edge cases not covered by Garvan's documentation

### Deferred Ideas (OUT OF SCOPE)
(None listed)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SIG-01 | `aqprod(a, q, n)` accepts Garvan's 3-arg signature | **Already implemented in Phase 33.** Garvan source confirms: `aqprod(a,q,n)` where a is symbolic, q is variable, n is integer. Our dispatch at eval.rs:1150-1200 already handles this. Phase 34 only needs help text update. |
| SIG-02 | `etaq(q, a, T)` matches Garvan's signature | **Partially implemented in Phase 33** (single delta). Garvan: `etaq(q,i,trunk)` -- q is variable, i is positive integer delta, trunk is truncation. Need to add multi-delta list form `etaq(q, [1,2,3], T)`. |
| SIG-03 | `jacprod(a, b, q, T)` matches Garvan's 4-arg signature | Garvan: `jacprod(a,b,q,T)` where a,b are integers, q is variable, T is truncation. Currently `jacprod(a,b,order)` with 3 args and implicit q. Add 4-arg form with Symbol detection. |
| SIG-04 | `tripleprod(a, q_power, T)` matches Garvan's signature with q-monomial | Garvan: `tripleprod(z,q,T)` where z is q-monomial (like q^3), q is variable (but is used as the base like q^b), T is truncation. Currently `tripleprod(cn,cd,p,order)` with 4 numeric args. Add 3-arg Maple form. |
| SIG-05 | `quinprod(a, q_power, T)` matches Garvan's signature | Garvan: `quinprod(z,q,T)` -- same pattern as tripleprod. Currently 4 numeric args. Add 3-arg Maple form. |
| SIG-06 | `winquist(a, b, q, T)` matches Garvan's signature | Garvan: `winquist(a,b,q,T)` where a,b are q-monomials, q is variable, T is truncation. Currently 7 numeric args. Add 4-arg Maple form. |
| SIG-07 | `qbin(n, k, q, T)` matches Garvan's signature with explicit q and T | Garvan: `qbin(q,m,n)` where q is variable, m and n are integers. Note: Garvan uses 3 args with q first, our requirement says 4 args. Disambiguation needed (see detailed analysis below). |
| SIG-26 | `numbpart(n)` is the primary name for partition counting | Maple `combinat[numbpart](n)` counts partitions. Optionally `numbpart(n,m)` with max part bound. Currently `numbpart` -> `partition_count` alias. Reverse: make `numbpart` primary, `partition_count` alias. |
</phase_requirements>

## Garvan's Exact Function Signatures (from source code)

### Source Verification

The definitive source is `wmprog64.txt` from qseries.org version 1.2, fetched and analyzed directly. All signatures below are verified HIGH confidence from the actual Maple source code.

### Signature Table

| Function | Garvan Maple Signature | Current q-Kangaroo Signature | Change Needed |
|----------|----------------------|------------------------------|---------------|
| `aqprod` | `aqprod(a, q, n)` -- a is symbolic (q^2, etc.), q is variable, n is integer | `aqprod(monomial, q, n)` or `aqprod(cn, cd, power, n_or_inf, order)` | **Already done in Phase 33.** Help text update only. |
| `etaq` | `etaq(q, i, trunk)` -- q is variable, i is positive integer (delta), trunk is truncation | `etaq(q, b, order)` (Maple) or `etaq(b, t, order)` (legacy) | **Partially done.** Add multi-delta `etaq(q, [deltas], T)`. |
| `jacprod` | `jacprod(a, b, q, T)` -- a,b integers, q variable, T truncation | `jacprod(a, b, order)` -- 3 args, implicit q | **New.** Add 4-arg form. |
| `tripleprod` | `tripleprod(z, q, T)` -- z is q-monomial expression, q is variable/q-power, T truncation | `tripleprod(cn, cd, power, order)` -- 4 numeric args | **New.** Add 3-arg form. |
| `quinprod` | `quinprod(z, q, T)` -- z is q-monomial, q is variable/q-power, T truncation | `quinprod(cn, cd, power, order)` -- 4 numeric args | **New.** Add 3-arg form. |
| `winquist` | `winquist(a, b, q, T)` -- a,b q-monomials, q variable, T truncation | `winquist(a_cn,a_cd,a_p,b_cn,b_cd,b_p,order)` -- 7 numeric args | **New.** Add 4-arg form. |
| `qbin` | `qbin(q, m, n)` -- q is variable, m and n are integers (computes q-binomial [n choose m]) | `qbin(n, k, order)` -- 3 integer args, implicit q | **New.** Add Maple-style form. |
| `numbpart` | `numbpart(n)` or `numbpart(n, m)` -- from Maple's combinat package, NOT Garvan's qseries | `numbpart` is alias for `partition_count` | **Reverse alias direction.** |

### Detailed Signature Analysis

#### aqprod(a, q, n) -- SIG-01
- **Garvan source:** `proc(a,q,n)` -- `a` is any symbolic expression (q^2, 1, -q^3, etc.), `q` is the symbolic variable, `n` is a non-negative integer
- **Phase 33 already implements this.** The Maple-style path at eval.rs:1150-1200 detects first arg as Series/Symbol, extracts monomial, extracts symbol_id for q, and dispatches correctly.
- **Remaining work:** Update help text to show Maple signature as primary. Remove legacy signature from help display.
- **Disambiguation:** First-arg type (Series/Symbol -> Maple, Integer -> legacy). Already working.

#### etaq(q, i, trunk) -- SIG-02
- **Garvan source:** `proc(q,i,trunk)` -- `q` is variable symbol, `i` is positive integer (the "delta" divisor), `trunk` is truncation order
- **Phase 33 already handles single-delta:** `etaq(q, 1, 20)` works via first-arg Symbol detection at eval.rs:1213.
- **Multi-delta `etaq(q, [1,2,3], T)`:** NOT in Garvan's source code. The Garvan etaq only accepts a single integer `i`. However, the CONTEXT.md decision requires implementing this as syntactic sugar.
- **Implementation:** When second arg is `Value::List`, iterate the list, call `etaq(b_i, 1, sym, order)` for each delta, and multiply the results together. This produces the eta-quotient `prod_i etaq(q, delta_i, T)`.
- **Validation for lists:** If any element is not a positive integer, return an error. Garvan's `etaq` requires `i > 0` (otherwise product is zero for b<=0 in our Rust code).
- **Note on Garvan etaq semantics:** Garvan's etaq(q,i,T) computes the Euler product using pentagonal number theorem: `sum_{k=-z}^{z} (-1)^k * q^{i*k*(3k-1)/2}`. Our Rust etaq(b, t, sym, order) computes `prod_{n>=0}(1 - q^{b+t*n})`. When t=1, `etaq(b, 1, sym, T)` = `(q^b; q^b)_inf`, which equals Garvan's `etaq(q, b, T)` (both produce the Euler-type product). The mapping `etaq(q, i, T)` -> Rust `etaq(i, 1, sym, T)` is already correct from Phase 33 (but note: Garvan's `etaq(q,i,T)` actually uses the *pentagonal number expansion* to compute the q^i partition of the Euler product, not a sequential product - the mathematical result is identical).
- **Disambiguation:** First-arg type: Symbol -> Maple, Integer -> legacy.

#### jacprod(a, b, q, T) -- SIG-03
- **Garvan source:** `proc(a,b,q,T)` -- internally calls `tripleprod(q^a, q^b, T) / tripleprod(q^b, q^(3*b), T)`.
- **Wait:** This is the key insight. Garvan's `jacprod(a,b,q,T)` uses `tripleprod(q^a, q^b, T)`. Here `a` and `b` are integers (exponents), `q` is the symbolic variable, `T` is truncation. Our Rust `jacprod(a, b, variable, truncation_order)` already takes `a` and `b` as integers and builds `(q^a; q^b)_inf * (q^{b-a}; q^b)_inf * (q^b; q^b)_inf` which is the JAC(a,b) product. However, Garvan's jacprod actually divides two tripleprod calls, which is a different computation from our JAC(a,b). Let me verify...
- **Garvan's JAC(a,b):** `tripleprod(q^a, q^b, T) / tripleprod(q^b, q^(3*b), T)`. The tripleprod(z,q,T) computes `sum_{i} (-1)^i * z^i * q^{i*(i-1)/2}` (Jacobi's theta function). So Garvan's jacprod divides two theta-type sums.
- **Our JAC(a,b):** `(q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf` -- the infinite product form of the Jacobi triple product. By Jacobi's triple product theorem, `tripleprod(z,q,T) = (z;q)_inf * (q/z;q)_inf * (q;q)_inf`. Setting z=q^a, base=q^b: `tripleprod(q^a, q^b, T) = (q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf = JAC(a,b)`.
- And the denominator `tripleprod(q^b, q^{3b}, T) = (q^b;q^{3b})_inf * (q^{2b};q^{3b})_inf * (q^{3b};q^{3b})_inf = JAC(b,3b)`.
- So Garvan's `jacprod(a,b,q,T) = JAC(a,b) / JAC(b,3b)`. This is NOT the same as our `jacprod(a,b,sym,T) = JAC(a,b)`.
- **CRITICAL FINDING:** There is a mathematical discrepancy. Our Rust `jacprod(a,b,sym,T)` computes `JAC(a,b)`, while Garvan's `jacprod(a,b,q,T)` computes `JAC(a,b) / JAC(b,3b)`. This needs careful handling.
- **Wait -- let me re-read the Garvan source more carefully.** The Garvan definition is: `tripleprod(q^a,q^b,T)/tripleprod(q^(b),q^(3*b),T)`. Here `tripleprod` is being called with two q-expression arguments plus T. In Garvan's tripleprod, the first arg `z` is the "a" parameter and the second arg is the "q" parameter (the step). So `tripleprod(q^a, q^b, T)` means z=q^a, step=q^b. The Jacobi triple product with these params is `sum_i (-1)^i * (q^a)^i * (q^b)^{i*(i-1)/2}` = `sum_i (-1)^i * q^{ai + bi(i-1)/2}`.
- Actually wait -- Garvan's tripleprod computes: `sum_{i from -lasti to lasti} (-1)^i * z^i * q^{i*(i-1)/2}`. When called as `tripleprod(q^a, q^b, T)`, the `z` = q^a and the `q` = q^b. So the sum becomes: `sum_i (-1)^i * (q^a)^i * (q^b)^{i*(i-1)/2}` = `sum_i (-1)^i * q^{ai + b*i*(i-1)/2}`.
- And the denominator `tripleprod(q^b, q^{3b}, T)` = `sum_i (-1)^i * q^{bi + 3b*i*(i-1)/2}`.
- This is a different mathematical formula from what we compute. **Our current jacprod implementation is mathematically different from Garvan's jacprod.**
- **Resolution:** For the Maple-style `jacprod(a, b, q, T)`, we should implement Garvan's exact formula: compute `tripleprod(q^a, q^b, T) / tripleprod(q^b, q^{3b}, T)` using our tripleprod infrastructure. The legacy 3-arg `jacprod(a, b, order)` continues to use our existing JAC(a,b) formula.
- **ACTUALLY:** Let me reconsider. Looking at the success criterion: "`jacprod(1, 5, q, 30)` returns correct results". Let me check what Garvan's `jacprod(1,5,q,30)` produces. `tripleprod(q^1, q^5, 30) / tripleprod(q^5, q^15, 30)`. And from the Jacobi triple product identity, `tripleprod(q^a, q^b, T)` where the second arg is the nome should give the standard J(a,b). So `tripleprod(q^1, q^5, T)` = J(1,5) and `tripleprod(q^5, q^15, T)` = J(5,15) = J(5,15). Hmm, this division is unusual.
- **Let me verify with known values.** The Jacobi triple product identity states that `(z;q)_inf * (q/z;q)_inf * (q;q)_inf = sum_{n=-inf}^{inf} (-1)^n * z^n * q^{n(n-1)/2}`. So in Garvan's notation, `tripleprod(z,q,T)` IS the theta function `theta(z;q)`. And `jacprod(a,b,q,T)` = `theta(q^a; q^b) / theta(q^b; q^{3b})`.
- This is indeed a specific mathematical function. For the Phase 34 implementation, the Maple-style `jacprod(a, b, q, T)` must replicate Garvan's exact formula. We can implement it by calling `tripleprod` twice and dividing.
- **Confidence:** HIGH -- verified from source code.
- **Disambiguation:** Arg count: 4 args with Symbol at position 2 -> Maple, 3 integer args -> legacy.

#### tripleprod(z, q, T) -- SIG-04
- **Garvan source:** `proc(z, q, T)` -- `z` is a symbolic q-expression (like q^3, -q^2, z^2*q, etc.), `q` is the variable/nome, `T` is a positive integer truncation order.
- **Garvan computes:** `sum_{i} (-1)^i * z^i * q^{i*(i-1)/2}` where the sum runs from -lasti to lasti.
- **Our Rust tripleprod:** Takes a QMonomial z and computes the infinite product `(z;q)_inf * (q/z;q)_inf * (q;q)_inf` which by Jacobi's identity equals the same theta sum.
- **Note on Garvan's second argument:** In Garvan's `tripleprod(z, q, T)`, the `q` in the proc args shadows the `q` symbol. The `q^{i*(i-1)/2}` in the body uses this `q` parameter. So when called as `tripleprod(q^3, q, T)`, z=q^3 and the nome is q itself. When called as `tripleprod(q^a, q^b, T)` (from jacprod), z=q^a and the nome is q^b.
- **CRITICAL INSIGHT:** Garvan's tripleprod(z, q, T) treats `q` as the nome (base variable). If called with q=q (the symbol), it's standard. If called with q=q^b, the exponents scale by b. Our Rust tripleprod doesn't support a variable nome base -- it always uses the SymbolId directly with step=1.
- **For user-facing CLI:** `tripleprod(q^3, q, 20)` means z=q^3, nome=q, T=20. This maps to our Rust `tripleprod(QMonomial(1, 3), sym_q, 20)` perfectly. Users won't call `tripleprod(q^a, q^b, T)` directly -- that's internal to Garvan's jacprod. So for the CLI Maple-style signature: `tripleprod(z, q, T)` where z is a monomial and q is a Symbol, maps cleanly.
- **Disambiguation:** 3 args where first is Series/Symbol and second is Symbol -> Maple; 4 integer args -> legacy.

#### quinprod(z, q, T) -- SIG-05
- **Garvan source:** `proc(z, q, T)` -- identical structure to tripleprod.
- **Garvan computes:** `sum_i ((-z)^{-3i} - (-z)^{3i+1}) * q^{i*(3i+1)/2}`.
- **Our Rust quinprod:** Infinite product form that equals the same sum by the quintuple product identity.
- **Same mapping as tripleprod:** `quinprod(q^3, q, 20)` -> Rust `quinprod(QMonomial(1,3), sym_q, 20)`.
- **Disambiguation:** Same as tripleprod: 3 args with monomial+symbol -> Maple; 4 integer args -> legacy.

#### winquist(a, b, q, T) -- SIG-06
- **Garvan source:** `proc(a, b, q, T)` -- `a` and `b` are symbolic q-expressions (q-monomials), `q` is the variable, `T` is truncation.
- **Garvan computes:** Double sum over i,j with `a^{exponent} * b^{exponent} * q^{...}`.
- **Our Rust winquist:** `winquist(a: &QMonomial, b: &QMonomial, variable: SymbolId, truncation_order: i64)` -- already takes two QMonomials, a SymbolId, and truncation. Perfect match.
- **Maple-style mapping:** `winquist(q^1, q^2, q, 10)` -> extract monomials for a and b, extract symbol for q, use last arg as T.
- **Disambiguation:** 4 args where first two are monomial/symbol and third is symbol -> Maple; 7 integer args -> legacy.

#### qbin(q, m, n) -- SIG-07
- **Garvan source:** `proc(q, m, n)` -- `q` is the variable, `m` and `n` are non-negative integers. Computes `aqprod(q,q,n) / (aqprod(q,q,m) * aqprod(q,q,n-m))`.
- **Note:** Garvan's `qbin(q, m, n)` computes the q-binomial coefficient `[n choose m]_q`. The parameters are `q` (variable), `m` (top of the choose), `n` (bottom). Wait -- actually, looking at the formula: `aqprod(q,q,n)/aqprod(q,q,m)/aqprod(q,q,n-m)`. This is `(q;q)_n / ((q;q)_m * (q;q)_{n-m})` which is the Gaussian binomial `[n choose m]_q`. So m <= n.
- **Our legacy:** `qbin(n, k, order)` computes the q-binomial [n choose k]_q with implicit q and explicit truncation order. The parameters are n, k (choose), and truncation.
- **IMPORTANT difference:** Garvan's `qbin(q, m, n)` produces an exact polynomial (no truncation parameter) because it's a finite product. Our legacy `qbin(n, k, order)` has an explicit `order` parameter for truncation.
- **The requirement says:** `qbin(n, k, q, T)` -- this has 4 args with explicit q and T. This doesn't exactly match Garvan's 3-arg `qbin(q, m, n)`. Let me reconcile:
  - Garvan: `qbin(q, m, n)` -- 3 args, q first, no truncation (exact polynomial)
  - Requirement: `qbin(n, k, q, T)` -- 4 args, q third, T explicit
  - Our legacy: `qbin(n, k, order)` -- 3 args, all integers
- **Recommendation:** Implement BOTH:
  - Garvan exact: `qbin(q, m, n)` -- 3 args, first is Symbol, produces exact polynomial (POLYNOMIAL_ORDER sentinel)
  - With truncation: `qbin(n, k, q, T)` -- 4 args, Symbol at position 2, explicit truncation
  - Legacy: `qbin(n, k, order)` -- 3 args, all integers (existing behavior)
- **Disambiguation:** 3 args with Symbol first -> Garvan exact; 4 args with Symbol at index 2 -> Maple with truncation; 3 integer args -> legacy.

#### numbpart(n) and numbpart(n, m) -- SIG-26
- **Maple combinat[numbpart]:** `numbpart(n)` returns p(n); `numbpart(n, m)` returns number of partitions of n with max part m.
- **NOT in Garvan's qseries package** -- it's from Maple's built-in `combinat` module.
- **Current state:** `numbpart` is an alias for `partition_count` in resolve_alias (eval.rs:2633). `partition_count(n)` takes 1 arg.
- **Change needed:** Make `numbpart` the canonical name and `partition_count` the alias. Optionally add 2-arg form `numbpart(n, m)`.
- **For the 2-arg form:** This is equivalent to the number of partitions of n into parts each <= m. Our Rust core likely doesn't have this directly, but `bounded_parts_gf(m, T)` generates the GF for partitions with max part m. We could extract the coefficient, or implement it directly.
- **Decision:** Implement `numbpart(n)` as primary, `numbpart(n, m)` as bonus matching Maple. `partition_count` becomes alias.

## Architecture Patterns

### Pattern 1: Maple-style Dispatch (from Phase 33)
**What:** Detect Maple-style call by checking first argument type, then route to appropriate code path.
**When to use:** Every function that needs dual signatures.
**Example (already working for aqprod):**
```rust
"aqprod" => {
    // Detect Maple-style: first arg is Series (monomial like q^2) or Symbol
    if !args.is_empty() && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
        // Maple path: aqprod(a, q, n) or aqprod(a, q, n, order)
        let monomial = extract_monomial_from_arg(name, args, 0)?;
        let sym = extract_symbol_id(name, args, 1, env)?;
        // ...
    } else {
        // Legacy path: aqprod(coeff_num, coeff_den, power, n_or_infinity, order)
        expect_args(name, args, 5)?;
        // ...
    }
}
```

### Pattern 2: Help System Update
**What:** Change FuncHelp entries to show Maple signatures as primary.
**Location:** `crates/qsym-cli/src/help.rs` at FUNC_HELP array.
**Example:**
```rust
FuncHelp {
    name: "jacprod",
    signature: "jacprod(a, b, q, T)",  // Maple signature only
    description: "Compute the Jacobi product JAC(a,b,q) truncated to O(q^T).\n  Parameters: a, b are positive integers, q is the variable, T is truncation order.",
    example: "q> jacprod(1, 5, q, 30)",
    example_output: "...",
},
```

### Pattern 3: Alias Direction Reversal
**What:** Swap canonical/alias for numbpart.
**Location:** eval.rs resolve_alias, help.rs, repl.rs tab completion, ALL_FUNCTION_NAMES.
**Steps:**
1. Change resolve_alias: `"partition_count" => "numbpart"` (reverse of current)
2. Change dispatch: rename canonical from "partition_count" to "numbpart"
3. Update help: Move help entry name to "numbpart"
4. Update ALL_FUNCTION_NAMES: replace "partition_count" with "numbpart"
5. Update repl.rs: replace "partition_count" with "numbpart" in completions
6. Keep "partition_count" as alias name in ALL_ALIAS_NAMES
7. Keep "numbpart" in ALL_ALIAS_NAMES too (for backward compat -- or remove since it's now canonical)

### Anti-Patterns to Avoid

- **Breaking legacy signatures:** NEVER remove legacy dispatch paths. They must continue working silently.
- **Over-validating Maple args:** Garvan's Maple is dynamically typed; if a user passes something weird, it may produce garbage but not error. We should validate types (symbols/monomials) but not over-constrain.
- **Inconsistent disambiguation:** Don't use arg-count for one function and first-arg-type for another unless necessary. Prefer first-arg type detection consistently.
- **Modifying qsym-core:** This phase should only change qsym-cli. The Rust core functions already accept the right types (QMonomial, SymbolId, i64). All changes are in the dispatch layer.

## Disambiguation Strategy Per Function

| Function | Maple Detection | Legacy Detection |
|----------|----------------|------------------|
| aqprod | args[0] is Series/Symbol | args[0] is Integer (and 5 args) |
| etaq | args[0] is Symbol | args[0] is Integer |
| jacprod | len==4 AND args[2] is Symbol | len==3 AND all Integer |
| tripleprod | len==3 AND args[0] is Series/Symbol | len==4 AND all Integer |
| quinprod | len==3 AND args[0] is Series/Symbol | len==4 AND all Integer |
| winquist | len==4 AND args[2] is Symbol | len==7 AND all Integer |
| qbin | args[0] is Symbol (3 args) OR args[2] is Symbol (4 args) | len==3 AND all Integer |
| numbpart | N/A (replaces partition_count as canonical) | N/A |

## etaq Multi-Delta Implementation

### Design
When `etaq(q, [deltas], T)` is called:
1. Detect second arg is `Value::List`
2. For each delta in list, validate it's a positive integer
3. Compute `product_i etaq(delta_i, 1, sym, T)` by multiplying individual series
4. Return the product

### Example
```
etaq(q, [1, 2, 3], 20)
```
Computes: `etaq(q, 1, 20) * etaq(q, 2, 20) * etaq(q, 3, 20)`
= `(q;q)_inf * (q^2;q^2)_inf * (q^3;q^3)_inf` truncated at O(q^20)

### Validation
- Empty list: error
- Non-integer element: error
- Non-positive element: error (Garvan etaq requires i > 0)
- Single-element list `[3]`: equivalent to `etaq(q, 3, T)`

## jacprod Mathematical Correction

### Finding
Garvan's `jacprod(a,b,q,T)` = `tripleprod(q^a, q^b, T) / tripleprod(q^b, q^{3*b}, T)`.

This is NOT simply JAC(a,b). It's a ratio of two Jacobi theta functions.

### Impact
The Maple-style `jacprod(a, b, q, T)` must implement Garvan's exact formula. Implementation:
```rust
// Maple-style: jacprod(a, b, q, T)
let z1 = QMonomial::q_power(a_val);
let z2 = QMonomial::q_power(b_val);
let tp1 = qseries::tripleprod(&z1, sym, order); // But wait -- this uses step=1, not step=b
```

**ISSUE:** Our Rust `tripleprod(z, variable, T)` always uses step size 1 (q is the nome). Garvan's `tripleprod(q^a, q^b, T)` uses `q^b` as the nome, effectively computing with step size `b`. Our Rust function doesn't support a variable step size.

**Resolution options:**
1. Implement Garvan's jacprod formula directly using the theta sum: `sum_i (-1)^i * q^{a*i + b*i*(i-1)/2}` divided by `sum_i (-1)^i * q^{b*i + 3*b*i*(i-1)/2}`. This avoids needing a variable-step tripleprod.
2. Add a step-size parameter to tripleprod in qsym-core. Heavier change.
3. Use `etaq` building blocks: `tripleprod(q^a, q^b, T)` = `etaq(a, b, sym, T) * etaq(b-a, b, sym, T) * etaq(b, b, sym, T)`. Our Rust etaq already supports arbitrary b and t parameters!

**Recommended:** Option 3 -- use our existing `qseries::etaq(b, t, sym, T)` with t=b_val. For Garvan's `tripleprod(q^a, q^b, T)`:
- Factor 1: `etaq(a, b, sym, T)` = `(q^a; q^b)_inf`
- Factor 2: `etaq(b-a, b, sym, T)` = `(q^{b-a}; q^b)_inf`
- Factor 3: `etaq(b, b, sym, T)` = `(q^b; q^b)_inf`
- Product = JAC(a,b) with step b.

Then Garvan's `jacprod(a,b,q,T) = JAC(a,b) / JAC(b,3b)`.

So the Maple-style jacprod dispatch:
```rust
let jac_ab = jacprod_generalized(a_val, b_val, sym, order);
let jac_b_3b = jacprod_generalized(b_val, 3 * b_val, sym, order);
let result = arithmetic::div(&jac_ab, &jac_b_3b);
```

Where `jacprod_generalized(a, b, sym, T)` uses three etaq calls to compute `(q^a;q^b) * (q^{b-a};q^b) * (q^b;q^b)`.

**Wait -- our existing `qseries::jacprod(a, b, sym, T)` already does exactly this!** Looking at the source in products.rs:86-94:
```rust
pub fn jacprod(a: i64, b: i64, variable: SymbolId, truncation_order: i64) -> FormalPowerSeries {
    let p1 = etaq(a, b, variable, truncation_order);
    let p2 = etaq(b - a, b, variable, truncation_order);
    let p3 = etaq(b, b, variable, truncation_order);
    let temp = arithmetic::mul(&p1, &p2);
    arithmetic::mul(&temp, &p3)
}
```

So our Rust `jacprod(a, b, sym, T)` computes `(q^a;q^b)_inf * (q^{b-a};q^b)_inf * (q^b;q^b)_inf` = JAC(a,b).

And Garvan's `jacprod(a,b,q,T)` = `JAC(a,b) / JAC(b,3b)`.

So the Maple dispatch for jacprod is:
```rust
let jac_ab = qseries::jacprod(a_val, b_val, sym, order);
let jac_b3b = qseries::jacprod(b_val, 3 * b_val, sym, order);
let result = arithmetic::div(&jac_ab, &jac_b3b);
```

This is straightforward and uses existing core functions.

## numbpart Implementation

### Single-arg: numbpart(n)
Identical to current `partition_count(n)` -- calls `qseries::partition_count(n)`.

### Two-arg: numbpart(n, m)
Computes partitions of n with max part <= m. Implementation options:
1. Use `bounded_parts_gf(m, n+1)` and extract coefficient of q^n
2. Implement directly with a DP algorithm

Option 1 is cleanest since `bounded_parts_gf` already exists. Extract the coefficient at position n from the generating function.

## Output Format Analysis

### Coefficient Formatting
Our current format (format.rs) produces:
- `2*q^3` (with explicit `*`)
- `q^2` (coefficient 1 is implicit)
- `-q^3` (negative coefficient)
- `O(q^20)` (truncation)
- Ascending power order (BTreeMap iteration)

Garvan's Maple output (from search results and tutorial examples):
- `1 - q^2 - 2*q^3 + q^5 + q^7 + O(q^11)` (with `*` between coefficient and variable)
- Ascending power order
- Uses `O(q^T)` notation

**Conclusion:** Our format already matches Garvan's. Both use `coeff*q^power` with explicit `*` and ascending powers. No format changes needed.

### Polynomial Display
For exact polynomials (like qbin results), our POLYNOMIAL_ORDER sentinel suppresses `O(...)`. Garvan's Maple also displays exact polynomials without O notation. Already matches.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Garvan jacprod formula | Custom theta sum computation | `qseries::jacprod(a,b,sym,T)` + `arithmetic::div` | Existing core already computes JAC(a,b) correctly |
| Multi-delta etaq | New core function | Multiply individual `qseries::etaq` calls | Composition is simpler and tested |
| numbpart(n,m) | Custom partition counting | `qseries::bounded_parts_gf(m, n+1)` + coefficient extraction | Reuses existing GF computation |
| Monomial extraction | Manual parsing | `extract_monomial_from_arg` from Phase 33 | Already handles Symbol, Series, Integer |

## Common Pitfalls

### Pitfall 1: jacprod Maple vs Legacy Semantic Mismatch
**What goes wrong:** Implementing Maple jacprod(a,b,q,T) as just a wrapper for the legacy 3-arg form, when Garvan's jacprod actually computes JAC(a,b)/JAC(b,3b).
**Why it happens:** Assuming the name implies the same math.
**How to avoid:** Maple jacprod dispatches to `jacprod(a,b,sym,T) / jacprod(b,3b,sym,T)`. Test against Garvan's output.
**Warning signs:** Coefficients don't match Garvan for jacprod(1,5,q,30).

### Pitfall 2: qbin Argument Order Confusion
**What goes wrong:** Getting confused between Garvan's `qbin(q,m,n)` and the requirement's `qbin(n,k,q,T)` and the legacy `qbin(n,k,order)`.
**Why it happens:** Three different conventions for the same function.
**How to avoid:** Disambiguate by first-arg type AND arg count. Write explicit tests for each form.
**Warning signs:** qbin results differ between old and new calls.

### Pitfall 3: etaq Multi-Delta Truncation Accumulation
**What goes wrong:** Multiplying several etaq series with different effective truncation orders, getting wrong truncation in the result.
**Why it happens:** Each individual etaq call truncates at T, but the product might need higher internal precision.
**How to avoid:** All individual etaq calls use the same truncation T. The product's truncation is min of inputs, which is T since all are the same.
**Warning signs:** Last few coefficients of multi-delta product differ from expected.

### Pitfall 4: Alias Reversal Breaking Tests
**What goes wrong:** Changing numbpart from alias to primary name breaks existing tests that expect `partition_count` as canonical.
**Why it happens:** Tests may check resolve_alias, function dispatch, or help text with specific names.
**How to avoid:** Systematically update all 5 locations: resolve_alias, dispatch match arm, help.rs, ALL_FUNCTION_NAMES, repl.rs. Run full test suite.
**Warning signs:** Tests referencing "partition_count" fail after renaming.

### Pitfall 5: tripleprod/quinprod Disambiguation Overlap
**What goes wrong:** A 3-arg call where first arg is an integer (like `tripleprod(1, 1, 20)`) might ambiguously match either legacy (as 3 of 4 args) or fail.
**Why it happens:** Legacy tripleprod is 4 args, Maple is 3 args. There's no overlap in arg count.
**How to avoid:** Use arg count as primary discriminator: 3 args -> Maple (if types match); 4 args -> legacy. This is clean because the arg counts don't overlap.

## Files That Need Changes

| File | Changes |
|------|---------|
| `crates/qsym-cli/src/eval.rs` | Add Maple-style dispatch paths for jacprod, tripleprod, quinprod, winquist, qbin; add etaq multi-delta; rename partition_count canonical to numbpart; update resolve_alias; update get_signature; update ALL_FUNCTION_NAMES |
| `crates/qsym-cli/src/help.rs` | Update FUNC_HELP signatures and examples for all 7 functions; rename partition_count entry to numbpart; add partition_count redirect |
| `crates/qsym-cli/src/repl.rs` | Update canonical_function_names: replace partition_count with numbpart |
| `crates/qsym-cli/tests/cli_integration.rs` | Add integration tests for all new Maple-style signatures; test numbpart as primary name |

## Code Examples

### jacprod Maple-style dispatch
```rust
"jacprod" => {
    if args.len() == 4 && matches!(&args[2], Value::Symbol(_)) {
        // Maple: jacprod(a, b, q, T)
        let a_val = extract_i64(name, args, 0)?;
        let b_val = extract_i64(name, args, 1)?;
        let sym = extract_symbol_id(name, args, 2, env)?;
        let order = extract_i64(name, args, 3)?;
        // Garvan: jacprod(a,b,q,T) = tripleprod(q^a,q^b,T)/tripleprod(q^b,q^{3b},T)
        // = JAC(a,b) / JAC(b,3b)
        let jac_ab = qseries::jacprod(a_val, b_val, sym, order);
        let jac_b3b = qseries::jacprod(b_val, 3 * b_val, sym, order);
        let result = arithmetic::div(&jac_ab, &jac_b3b);
        Ok(Value::Series(result))
    } else {
        // Legacy: jacprod(a, b, order)
        expect_args(name, args, 3)?;
        let a = extract_i64(name, args, 0)?;
        let b = extract_i64(name, args, 1)?;
        let order = extract_i64(name, args, 2)?;
        let result = qseries::jacprod(a, b, env.sym_q, order);
        Ok(Value::Series(result))
    }
}
```

### tripleprod Maple-style dispatch
```rust
"tripleprod" => {
    if args.len() == 3 && matches!(&args[0], Value::Series(_) | Value::Symbol(_)) {
        // Maple: tripleprod(z, q, T) -- z is monomial, q is variable
        let monomial = extract_monomial_from_arg(name, args, 0)?;
        let sym = extract_symbol_id(name, args, 1, env)?;
        let order = extract_i64(name, args, 2)?;
        let result = qseries::tripleprod(&monomial, sym, order);
        Ok(Value::Series(result))
    } else {
        // Legacy: tripleprod(cn, cd, power, order)
        expect_args(name, args, 4)?;
        // ... existing code ...
    }
}
```

### etaq multi-delta dispatch
```rust
"etaq" => {
    if args.len() >= 2 && matches!(&args[0], Value::Symbol(_)) {
        let sym = extract_symbol_id(name, args, 0, env)?;
        if args.len() == 3 && matches!(&args[1], Value::List(_)) {
            // Multi-delta: etaq(q, [d1, d2, ...], T)
            let deltas = extract_i64_list(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let mut result = FormalPowerSeries::one(sym, order);
            for d in deltas {
                if d <= 0 {
                    return Err(EvalError::Generic { message: "etaq: delta must be positive".into() });
                }
                let factor = qseries::etaq(d, 1, sym, order);
                result = arithmetic::mul(&result, &factor);
            }
            Ok(Value::Series(result))
        } else {
            // Single delta: etaq(q, b, T)
            expect_args(name, args, 3)?;
            let b = extract_i64(name, args, 1)?;
            let order = extract_i64(name, args, 2)?;
            let result = qseries::etaq(b, 1, sym, order);
            Ok(Value::Series(result))
        }
    } else {
        // Legacy: etaq(b, t, order)
        // ... existing code ...
    }
}
```

## Test Vectors

### Success Criteria Verification

| Criterion | Function Call | Expected Output |
|-----------|-------------|-----------------|
| SC-1 | `aqprod(q^2, q, 5)` | Exact polynomial matching `(q^2;q)_5` = `(1-q^2)(1-q^3)(1-q^4)(1-q^5)(1-q^6)` |
| SC-2 | `etaq(q, 3, 20)` | `(q^3;q^3)_inf` truncated at O(q^20) |
| SC-3 | `jacprod(1, 5, q, 30)` | `JAC(1,5)/JAC(5,15)` truncated at O(q^30) |
| SC-3 | `qbin(4, 2, q, 10)` | q-binomial [4 choose 2]_q (should be a polynomial) |
| SC-4 | `numbpart(100)` | 190569292 |
| SC-5 | `tripleprod(q^3, q, 20)` | Jacobi triple product with z=q^3 |
| SC-5 | `quinprod(q^2, q, 20)` | Quintuple product with z=q^2 |
| SC-5 | `winquist(q, q^2, q, 10)` | Winquist product with a=q, b=q^2 |

### Additional test vectors
- `etaq(q, [1,2,3], 10)` -- multi-delta product
- `numbpart(5)` == 7
- `numbpart(50)` == 204226
- Legacy forms still work: `etaq(1, 1, 20)`, `jacprod(1, 5, 20)`, `tripleprod(1, 1, 1, 20)`

## Open Questions

1. **jacprod(a,b,q,T) = JAC(a,b)/JAC(b,3b) verification**
   - What we know: Garvan source clearly shows `tripleprod(q^a,q^b,T)/tripleprod(q^b,q^(3*b),T)`
   - What's unclear: Is this the universally expected definition, or might some users expect plain JAC(a,b)?
   - Recommendation: Implement Garvan's exact formula for Maple-style. Legacy keeps plain JAC(a,b). Document the difference in help text if needed.

2. **qbin(q, m, n) vs qbin(n, k, q, T)**
   - What we know: Garvan uses `qbin(q, m, n)` with 3 args; requirement says `qbin(n, k, q, T)` with 4 args.
   - What's unclear: Should we support both 3-arg Garvan and 4-arg variant?
   - Recommendation: Support 3-arg Garvan `qbin(q, m, n)` -> exact polynomial, and keep legacy `qbin(n, k, order)`. The 4-arg form `qbin(n, k, q, T)` can also be supported with explicit truncation.

3. **numbpart(n, m) bounded partition counting**
   - What we know: Maple supports it; our bounded_parts_gf exists
   - What's unclear: Whether to implement the 2-arg form in this phase
   - Recommendation: Implement 2-arg form since it's part of "matching full Maple signature" per CONTEXT.md

## Sources

### Primary (HIGH confidence)
- Garvan qseries source code: `wmprog64.txt` from https://qseries.org/fgarvan/qmaple/qseries/1.2/maple16-win64/ -- all function signatures verified directly from source
- Maple combinat[numbpart] documentation: https://www.maplesoft.com/support/help/Maple/view.aspx?path=combinat/numbpart
- Codebase analysis: `crates/qsym-cli/src/eval.rs`, `crates/qsym-core/src/qseries/products.rs`, `crates/qsym-core/src/qseries/pochhammer.rs`

### Secondary (MEDIUM confidence)
- Garvan q-product tutorial: https://qseries.org/fgarvan/papers/qmaple.pdf (PDF, not directly parseable but cross-referenced with source)
- Updated qseries documentation: https://qseries.org/fgarvan/qmaple/qseries/doc/qseriesdoc.pdf
- ArXiv paper: https://arxiv.org/abs/math/9812092

### Tertiary (LOW confidence)
- GitHub mirror: https://github.com/tarregahong/Maple_Package (could not access tree directly)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries, all changes are in CLI dispatch layer
- Architecture: HIGH -- Phase 33 established the pattern, this phase extends it mechanically
- Pitfalls: HIGH -- all identified from actual source code comparison and existing test analysis
- jacprod formula: HIGH -- verified directly from Garvan source code, confirmed by product identity analysis
- qbin argument order: MEDIUM -- Garvan source is definitive, but requirement text says 4 args while Garvan uses 3

**Research date:** 2026-02-19
**Valid until:** 2026-03-19 (stable domain, no external dependencies changing)
