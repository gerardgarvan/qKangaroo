# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 8 -- Mock Theta & Bailey Chains

## Current Position

Phase: 8 of 8 (Mock Theta & Bailey Chains) -- IN PROGRESS
Plan: 3 of 4 in current phase (3 complete: 08-01, 08-02, 08-03)
Status: Executing Phase 8; 08-03 (Bailey pairs) complete
Last activity: 2026-02-14 -- Completed 08-03-PLAN.md

Progress: [#####################] 100%

## Performance Metrics

**Velocity:**
- Total plans completed: 31
- Average duration: 7 min
- Total execution time: 3.6 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 - Expression Foundation | 3/3 | 37 min | 12 min |
| 2 - Simplification & Series Engine | 3/3 | 14 min | 5 min |
| 3 - Core q-Series & Partitions | 4/4 | 11 min | 3 min |
| 4 - Series Analysis | 7/7 | 57 min | 8 min |
| 5 - Python API | 4/4 | 20 min | 5 min |
| 6 - Hypergeometric Series | 4/4 | 35 min | 9 min |
| 7 - Identity Proving | 4/4 | 25 min | 6 min |
| 8 - Mock Theta & Bailey Chains | 3/4 | 24 min | 8 min |

**Recent Trend:**
- Last 5 plans: 5 min, 5 min, 8 min, 8 min, 8 min
- Trend: stable ~7 min/plan

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 8 phases derived from 62 v1 requirements; strict linear dependency chain
- [Roadmap]: Python API placed after qseries parity (Phase 5) so Rust API stabilizes first
- [Roadmap]: Partition basics (PART-01 through PART-03) grouped with core q-series (Phase 3) since they are natural applications of q-Pochhammer
- [Roadmap]: Mock theta and Bailey chains grouped together (Phase 8) as the most advanced extensions
- [01-01]: Manual Hash impl for QInt/QRat via to_digits() + sign (rug types may not impl Hash on all platforms)
- [01-01]: Kept Neg as separate Expr variant (not Mul([-1, x])) for Phase 1 simplicity
- [01-01]: ExprRef u32 numeric ordering for canonical sort (deterministic within session)
- [01-01]: Pre-built GMP/MPFR system libs via MSYS2 packages for Windows GNU target
- [01-01]: GNU toolchain (x86_64-pc-windows-gnu) required; MSVC target unsupported by gmp-mpfr-sys
- [01-02]: Division-by-zero panics (assert!) rather than Result -- matches rug and Rust Div convention
- [01-02]: Integer division is truncating (floor toward zero) per rug default and Rust convention
- [01-03]: Always-brace policy for LaTeX sub/superscripts to eliminate edge-case bugs
- [01-03]: ASCII fallback for non-numeric Unicode sub/superscripts (digits only get Unicode rendering)
- [01-03]: Neg detection in Add: renders as subtraction (a - b) not addition of negative (a + -b)
- [02-01]: Hardcoded 'q' as display variable name in FPS Display impl -- no SymbolRegistry access; Phase 3+ can add display_with_arena
- [02-01]: Shift adjusts truncation_order by k (shift(f, k) has trunc = f.trunc + k)
- [02-01]: pub(crate) fields on FPS -- arithmetic accesses directly, external users use API
- [02-01]: PartialEq compares variable + truncation_order + coefficient maps (value equality)
- [02-02]: Direct Rust match arms for simplification rules (not generic pattern matcher) -- handles n-ary operators correctly
- [02-02]: 4 rule phases with restart-from-phase-1 on change, max 100 iterations for termination guarantee
- [02-02]: intern_numeric auto-promotes to Integer when QRat denominator is 1
- [02-03]: ensure_order uses initial truncation_order (not target_order) for factor construction -- prevents permanent truncation reduction on incremental reuse
- [02-03]: Use 'ipg' variable name for InfiniteProductGenerator instances ('gen' is a reserved keyword in Rust)
- [03-01]: QMonomial uses QRat coeff + i64 power (not generic Expr) -- keeps q-series layer simple and fast
- [03-01]: Negative order via shifted-a inversion: (a;q)_{-n} = 1/(a*q^{-n};q)_n, reusing finite positive
- [03-01]: qbin uses numerator/denominator product ratio with arithmetic::invert, not incremental geometric series
- [03-02]: All 5 product functions implemented together to satisfy module re-export compilation
- [03-02]: tripleprod/quinprod verified via Jacobi bilateral series identity rather than hand-computed coefficients
- [03-02]: winquist tested with rational QMonomial coefficients (1/3, 1/5) to avoid integer-offset vanishing edge cases
- [03-02]: Fixed qpochhammer_inf_generator exp==0 bug: set constant to (1-coeff) not -coeff for zero-exponent factors
- [03-03]: theta2 returned as series in X=q^{1/4} with integer exponents representing powers of q^{1/4}
- [03-03]: Shared q2_q2_inf helper extracted for (q^2;q^2)_inf factor common to theta3 and theta4
- [03-04]: rank_gf and crank_gf return partition_gf directly at z=1 to handle removable singularity
- [03-04]: odd_parts_gf uses explicit factor loop with inversion rather than qpochhammer_inf_generator with step parameter
- [04-01]: Automatic normalization in prodmake: strips min_order shift and scalar prefactor before Andrews' algorithm
- [04-01]: QRat exponents in InfiniteProductForm (not i64) to support fractional exponents in eta-quotients
- [04-01]: mobius/divisors as module-private helpers with co-located unit tests
- [04-02]: Top-down factoring order in qfactor: try largest (1-q^i) first to prevent subfactor stealing from smaller factors
- [04-02]: Polynomial degree bound check in division prevents infinite series expansion when input is not polynomial-divisible
- [04-02]: zqfactor (two-variable) left as stub per Garvan's unreliability documentation
- [04-03]: Fermat's little theorem for modular inverse (a^{p-2} mod p) rather than extended Euclidean
- [04-03]: i128 intermediates in mod_mul to prevent overflow in modular arithmetic
- [04-03]: Null space basis uses free-variable-equals-1 convention (canonical form)
- [04-04]: Mobius inversion for etamake: r_n = sum_{d|n} mu(n/d) * (-a_d) rather than iterative subtraction
- [04-04]: QRat-to-i64 conversion in eta/qeta factors via to_f64() cast -- valid for integer exponents
- [04-04]: Period search in jacprodmake tries all b from 1 to max_n, picking best coverage
- [04-05]: Monomial ordering in findhom uses generate_monomials lexicographic order (last index varies fastest)
- [04-05]: fps_pow uses repeated squaring for efficient power computation
- [04-05]: findlincombo normalizes null space vector so first component (for f) equals 1, then negates remaining
- [04-06]: findcong tests candidate divisors from fixed prime list [2,3,5,7,11,13,17,19,23,29,31] plus the modulus itself
- [04-06]: findnonhom concatenates monomials for each degree 0,1,...,d in order, reusing generate_monomials from Plan 05
- [04-06]: findhomcombo/findnonhomcombo prepend target f to candidate list, then normalize null space vector with nonzero f-component
- [04-07]: Local mod_inv_local/mod_pow_local helpers in relations.rs rather than importing from linalg to avoid pub exposure
- [04-07]: QRat-to-modp conversion via rug Integer::is_divisible check + Fermat inverse, returns None if denominator divisible by p
- [04-07]: findmaxind uses inline Gaussian elimination to directly extract pivot columns rather than full null space
- [04-07]: findprod uses brute-force odometer iteration over [-max_coeff, max_coeff]^k with prodmake integer-exponent check
- [05-01]: PyO3 0.23 with PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 for Python 3.14 support (0.23 max is 3.13)
- [05-01]: maturin mixed layout: native module _qsymbolic, Python package qsymbolic/ with re-exports
- [05-01]: use-system-libs GMP works correctly for cdylib linking on Windows/Cygwin (highest-risk item validated)
- [05-02]: Used intern_rat() instead of direct rug::Rational to avoid rug dependency in qsym-python
- [05-02]: Used std Hash trait on ExprRef for __hash__ since ExprRef.0 is pub(crate)
- [05-02]: Auto-register MinGW DLL directory in __init__.py for Windows GMP shared library loading
- [05-03]: QSeries owns FPS directly (not Arc<Mutex>) -- FPS is standalone computation result, not arena expression
- [05-03]: partition_count extracts QRat numerator as QInt for Python int conversion
- [05-03]: sift DSL named sift_fn in Rust, registered as 'sift' in Python via pyo3(name) attribute
- [05-03]: extract_fps_refs uses explicit lifetime 'a on both &'a [PyRef<'a, QSeries>] and return Vec<&'a FPS>
- [05-04]: dispatch_generator as standalone helper shared by generate() and batch_generate() to avoid duplication
- [05-04]: batch_generate holds session lock once for entire batch, not per iteration
- [05-04]: n=-1 sentinel for PochhammerOrder::Infinite in batch params (Vec<Vec<i64>> has no Option)
- [05-04]: Generator-only restriction enforced with descriptive PyValueError listing all 15 supported functions
- [06-01]: FPS-based term accumulation (not direct coefficient) for eval_phi: handles general QMonomial parameters correctly
- [06-01]: Single inversion per step: accumulate denominator factors, invert once, reduces inversions from (s+1) to 1
- [06-01]: Pole detection in eval_psi_negative: skip terms where Pochhammer at negative order has pole (a.coeff==1 && 0<a.power<=m)
- [06-01]: rug::Integer MulIncomplete completed explicitly for sqrt comparison (lazy types cannot compare directly)
- [06-02]: q^2-Pochhammer via manual factor loop (not etaq) for general-coefficient step-2 products in Kummer formula
- [06-02]: Dixon z formula uses q^{2-n}/(bc) per DLMF 17.7.6 convention
- [06-02]: Terminating summation tests verify against product formula directly (not eval_phi) due to negative-power FPS limitation
- [06-02]: qrat_pow helper for QRat exponentiation via simple loop multiplication
- [06-03]: Sears permutation search: try 3*3=9 (a,d) role assignments to find balanced configuration automatically
- [06-03]: Sears test uses structural + prefactor verification (not eval_phi) due to FPS negative-power limitation on terminating series
- [06-03]: Heine tests use eval_phi expansion comparison directly since non-terminating 2phi1 with positive-power params is exact in FPS
- [06-04]: Watson test uses structural + prefactor verification (not eval_phi) due to FPS negative-power limitation on def/a parameter
- [06-04]: Bailey implemented as standalone function (not pattern-matching) because q^2 base requires different Pochhammer evaluation
- [06-04]: Python phi/psi use (num,den,power) tuple lists for QMonomial params -- more Pythonic than raw constructor calls
- [06-04]: Heine Python functions return (prefactor, combined_result) tuple where combined = prefactor * eval_phi(transformed)
- [07-01]: fps_pow shared in identity/mod.rs as pub(crate) rather than duplicated in jac.rs and eta.rs
- [07-01]: EtaExpression.from_etaquotient computes level as LCM of all deltas (EtaQuotient lacks level field)
- [07-01]: Newman condition 3 uses rug::Integer for perfect square check on prod(delta^|r_delta|)
- [07-01]: to_series panics on fractional q-shift (FPS only supports integer exponents)
- [07-02]: Cusp 0/1 generated by d=0 for c=1; infinity represents the c=N equivalence class
- [07-02]: Invariant order (cuspord) = sum gcd(c,delta)^2*r_delta/(24*delta) -- NOT the Ligozat formula with N/24 prefactor
- [07-02]: Weighted order (cuspORD) = cuspord * cusp_width; sum of cuspORDs = 0 for weight-0 eta quotients
- [07-03]: Empty combined factors (LHS=RHS) short-circuit to Proved without q-expansion to avoid non-integer q-shift panics
- [07-03]: Two-tier proving: structural valence formula for 2-term unit-coefficient identities, q-expansion fallback for all others
- [07-03]: Sturm bound computed for general weight; weight-0 with non-negative cusp orders checks constant term + 5-term safety margin
- [07-04]: toml 0.8 crate for TOML parsing (integrates with existing serde derive infrastructure)
- [07-04]: Embedded default database via include_str! for Python search_identities (no file path dependency)
- [07-04]: IdentitySide.factors uses BTreeMap<String, i64> for TOML key compatibility (delta parsed from string)
- [07-04]: Case-insensitive search across tags, functions, and patterns for user convenience
- [08-03]: Bailey pair alpha/beta terms return FPS (not QRat) since coefficients involve q-powers
- [08-03]: Rogers-Ramanujan a=1 limit form: alpha_n = (1+q^n)*(-1)^n*q^{n(3n-1)/2} (removable singularity)
- [08-03]: q-Binomial beta computed from defining relation (not closed form) for guaranteed correctness
- [08-03]: Tabulated pair stores Vec<FPS> for lemma-derived terms
- [08-03]: Bailey lemma params must avoid vanishing Pochhammer products (aq/b, aq/c not q^k for small k)

### Pending Todos

None yet.

### Blockers/Concerns

- [Resolved]: Andrews' algorithm (prodmake) + all 4 post-processing functions (etamake, jacprodmake, mprodmake, qetamake) now complete
- [Resolved]: Full QSER-19 relation discovery suite (12+ functions) now complete
- [Resolved]: Identity proving (Phase 7) complete with cusps, orders, valence formula, TOML database, Python bindings
- [Research]: Mock theta and Bailey chains (Phase 8) need algorithm extraction from academic literature
- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH. See .cargo/config.toml for env vars. Must use `export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"` before cargo commands.
- [Build]: PyO3 builds require PYO3_PYTHON pointing to Python 3.14 and PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1. maturin develop needs virtualenv at crates/qsym-python/.venv
- [03-02]: qpochhammer_inf_generator had exp==0 bug (now fixed); any pre-existing code using offset=0 with non-unity coefficient was affected

## Session Continuity

Last session: 2026-02-14
Stopped at: Completed 08-03-PLAN.md (Bailey pairs, lemma, chain, weak lemma)
Resume file: .planning/phases/08-mock-theta-bailey-chains/08-03-SUMMARY.md
