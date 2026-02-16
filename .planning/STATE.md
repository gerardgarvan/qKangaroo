# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-15)

**Core value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.
**Current focus:** v1.2 COMPLETE -- all 17 phases shipped

## Current Position

Phase: 17 of 17 (Python API & Documentation)
Plan: 2 of 2 in current phase (COMPLETE)
Status: v1.2 COMPLETE
Last activity: 2026-02-16 — Completed 17-02 (Identity Proving Extensions & Sphinx Summation)

Progress: [==================================================] 100% (v1.0+v1.1+v1.2 complete, all 17 phases shipped)

## v1.0 Performance Metrics

**Velocity:**
- Total plans completed: 32
- Average duration: 7 min
- Total execution time: 3.7 hours

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
| 8 - Mock Theta & Bailey Chains | 4/4 | 32 min | 8 min |

## v1.1 Performance Metrics

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 9 - Package Rename & Structure | 2/2 | 4 min | 2 min |
| 10 - PyPI Packaging & Metadata | 2/2 | 5 min | 2.5 min |
| 11 - CI/CD Pipeline | 2/2 | 2 min | 1 min |
| 12 - Documentation & UX Polish | 4/4 | 57 min | 14 min |

## v1.2 Performance Metrics

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 13 - Polynomial Infrastructure | 3/3 | 13 min | 4 min |
| 14 - q-Gosper Algorithm | 3/3 | 19 min | 6 min |
| 15 - q-Zeilberger & WZ Certificates | 3/3 | 62 min | 21 min |
| 16 - Extensions | 3/3 | 18 min | 6 min |
| 17 - Python API & Docs | 2/2 | 12 min | 6 min |

## Accumulated Context

### Decisions

All decisions logged in PROJECT.md Key Decisions table.
v1.0 decisions preserved in MILESTONES.md.
v1.1 decisions preserved in milestones/v1.1-ROADMAP.md.
- 13-01: Dense Vec<QRat> ascending-degree storage with trailing-zero normalization invariant
- 13-01: Content = gcd(numerators)/lcm(denominators) for rational coefficients
- 13-01: Four trait impl variants per arithmetic op matching QRat pattern
- 13-02: Subresultant PRS for GCD with content extraction before PRS loop
- 13-02: Euclidean algorithm (not subresultant) for resultant since Q[x] is a field
- 13-02: q_shift/q_shift_n as methods on QRatPoly for p(x)->p(q^j*x)
- 13-03: Auto-reduce via poly_gcd on every construction for canonical form at all times
- 13-03: Cross-cancellation in mul: gcd(a,d) and gcd(c,b) before multiplying
- 13-03: Monic denominator invariant; negation bypasses constructor (preserves invariants)
- 14-01: Redefine qrat_pow_i64 locally in gosper.rs (poly/mod.rs version is private)
- 14-01: q-dispersion upper bound deg(a)*deg(b) from resultant theory
- 14-01: q_dispersion_positive is pub(crate) for normal form decomposition
- 14-02: Telescoping product index range i=1..=j_max for correct c(qx)/c(x) identity
- 14-02: Private solve_linear_system (RREF over Q) rather than reusing rational_null_space
- 14-02: Degree bound fallback: try primary bound then primary+1 for q-power cancellation
- 14-03: Key equation RHS is tau(x)*c(x) for correct antidifference y(x)=f(x)/c(x)
- 14-03: Degree bound search extended to d_c+d_sigma+2 for cascading q-power cancellation
- 14-03: Certificate formula y(x)=f(x)/c(x) where s_k=y(q^k)*t_k satisfies S_{k+1}-S_k=t_k
- 15-01: Direct term-value approach instead of polynomial key equation evaluation for creative telescoping
- 15-01: Lagrange interpolation for WZ certificate construction from G(n,k) values
- 15-01: Duplicate private helpers from gosper.rs rather than making them pub(crate)
- 15-01: Boundary conditions G(n,0)=0 and G(n,max_k+1)=0 for telescoping sum
- 15-02: Include k=0 boundary in Lagrange interpolation for correct WZ certificate at R(1)=0
- 15-02: Return certificate directly from try_creative_telescoping (avoid double QRatRationalFunc::new reduction)
- 15-02: detect_n_params made fully public with documented limitations for non-standard series
- 15-03: WZ certificate verification skips termination boundary where G(n,k) != R(q^k)*F(n,k)
- 15-03: verify_recurrence_fps re-derives recurrence at each n (concrete-q coefficients are n-specific)
- 15-03: compute_sum_at_n uses direct term accumulation rather than eval_phi for concrete q
- 16-01: Characteristic polynomial + Rational Root Theorem for constant-coeff recurrences (not full q-Petkovsek normal form)
- 16-01: LCM normalization for QRat coefficients before rational root search
- 16-01: ClosedForm reserved for Pochhammer factorizations; pure q-power ratios return None
- 16-01: Divisor candidate cap at 5000 to prevent combinatorial explosion
- 16-03: BFS (not DFS) for shortest-path guarantee in transformation chains
- 16-03: Order-independent key normalization for visited-set dedup (sort param strings lexicographically)
- 16-03: FPS-based match condition: eval_phi(chain_end) == eval_phi(target)
- 16-03: Five-transformation catalog: heine_1, heine_2, heine_3, sears, watson
- 16-02: Scalar (QRat) rhs_builder interface for prove_nonterminating (avoids eval_phi negative-power bug)
- 16-02: Re-derive recurrence at each verification n (concrete-q coefficients are n-specific)
- 16-02: compute_sum_at_q for scalar sum evaluation via term-ratio accumulation
- 17-01: verify_wz_fn internally calls q_zeilberger (QRatRationalFunc not passable from Python)
- 17-01: q_petkovsek_fn takes (int,int) tuples for coefficients (QRat boundary pattern)
- 17-01: Option<Vec<usize>> + Option<bool> for auto-detect with manual override
- 17-02: Closure-from-template design for prove_nonterminating (declarative Python params, Rust builds closures)
- 17-02: pochhammer_scalar_val helper computes (q^b;q)_n at concrete q for RHS builder
- 17-02: 13 functional groups (not 12) reflecting q-Gosper as separate Group 11

### Pending Todos

None.

### Blockers/Concerns

- (RESOLVED) Research flag: subresultant PRS coefficient growth empirically verified on degree-10 polynomials in 13-02
- (RESOLVED) Research flag: qGFF implementation details resolved during Phase 14 planning and execution

## Session Continuity

Last session: 2026-02-16
Stopped at: Completed 17-02-PLAN.md (Identity Proving Extensions & Sphinx Summation). v1.2 milestone complete.
Resume file: N/A -- all plans complete
