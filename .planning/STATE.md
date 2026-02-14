# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-13)

**Core value:** Every function in Garvan's Maple packages works correctly in Q-Symbolic, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** Phase 4 -- Series analysis (in progress)

## Current Position

Phase: 4 of 8 (Series Analysis)
Plan: 4 of 7 in current phase (4 complete, including 04-01, 04-02, 04-03)
Status: In progress
Last activity: 2026-02-14 -- Completed 04-02-PLAN.md

Progress: [########..] 48%

## Performance Metrics

**Velocity:**
- Total plans completed: 12
- Average duration: 8 min
- Total execution time: 1.6 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 - Expression Foundation | 3/3 | 37 min | 12 min |
| 2 - Simplification & Series Engine | 3/3 | 14 min | 5 min |
| 3 - Core q-Series & Partitions | 4/4 | 11 min | 3 min |
| 4 - Series Analysis | 4/7 | 36 min | 9 min |

**Recent Trend:**
- Last 5 plans: 7 min, 4 min, 4 min, 10 min, 14 min
- Trend: increasing slightly ~8 min/plan

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

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: Andrews' algorithm (prodmake) implemented in 04-01; etamake/jacprodmake post-processing pending in 04-04
- [Research]: Identity proving (Phase 7) needs deep research on cusp theory and valence formula
- [Research]: Mock theta and Bailey chains (Phase 8) need algorithm extraction from academic literature
- [Build]: Windows build requires MinGW GCC 14.2.0 + pre-built GMP in PATH. See .cargo/config.toml for env vars. Must use `export PATH="/c/mingw64-gcc/mingw64/bin:/c/cygwin64/bin:/c/Users/Owner/.cargo/bin:$PATH"` before cargo commands.
- [03-02]: qpochhammer_inf_generator had exp==0 bug (now fixed); any pre-existing code using offset=0 with non-unity coefficient was affected

## Session Continuity

Last session: 2026-02-14
Stopped at: Completed 04-02-PLAN.md (qfactor, sift, qdegree, lqdegree with 13 tests, 311 total passing)
Resume file: .planning/phases/04-series-analysis/04-02-SUMMARY.md
