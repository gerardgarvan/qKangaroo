# Research Summary: Algorithmic q-Hypergeometric Identity Proving

**Domain:** q-Gosper, q-Zeilberger, creative telescoping, WZ proof certificates
**Researched:** 2026-02-15
**Overall confidence:** HIGH

## Executive Summary

This research investigates the stack, features, architecture, and pitfalls for adding algorithmic q-hypergeometric identity proving to q-Kangaroo. The target algorithms -- q-Gosper's algorithm (indefinite summation), q-Zeilberger's algorithm (creative telescoping for definite sums), and Wilf-Zeilberger proof certificates -- represent the gold standard for machine-proving identities involving basic hypergeometric series. These algorithms are well-established in the literature (Koornwinder 1993, Paule/Riese 1997, Petkovsek/Wilf/Zeilberger "A=B" 1996) with existing implementations in Maple (Koepf's qsum), Mathematica (Paule/Riese's qZeil), and Maxima (hipergeo). No Rust implementation exists.

The fundamental architectural challenge is that these algorithms require a **different computational substrate** than what q-Kangaroo currently has. The existing engine operates on formal power series (FPS) -- truncated numerical expansions in q. The new algorithms operate on symbolic polynomials and rational functions in variables like q^k, requiring polynomial GCD, resultant computation, polynomial equation solving via undetermined coefficients, and q-shift operator algebra. A new polynomial arithmetic layer must be built.

After evaluating all MIT-compatible Rust polynomial crates (feanor-math v3.5.13 -- slow for rationals per its own documentation; rust-poly v0.4.3 -- floating-point only) and ruling out license-incompatible ones (polynomial-ring and ring-algorithm -- AGPL; algebraics -- LGPL; Symbolica -- commercial), the recommendation is to **build a purpose-built polynomial module within qsym-core** (~650-890 lines). This reuses the existing QRat/QInt exact arithmetic, avoids license contamination, and allows optimization for the specific polynomial sizes (degree 5-30) that Gosper/Zeilberger produce. No new external dependencies are needed.

The algorithms themselves follow a clear dependency chain: polynomial arithmetic provides the foundation, q-Gosper builds on it as the core subroutine, q-Zeilberger calls q-Gosper iteratively for creative telescoping, and WZ certificates are extracted from Zeilberger's output. Total estimated new code is 1750-2250 lines, comparable to Phase 8 (mock theta + Appell-Lerch + Bailey = ~2020 lines).

## Key Findings

**Stack:** No new external dependencies. Build QRatPoly (dense univariate polynomial over QRat) and QRatRationalFunc (rational function type) in-house within qsym-core. All polynomial operations (GCD via subresultant PRS, resultant, division, shift, compose) use existing QRat exact arithmetic.

**Architecture:** Three-layer design -- (1) polynomial infrastructure (`poly/` module), (2) algorithm implementations (`gosper.rs`, `zeilberger.rs`), (3) proof artifacts (`wz.rs`). Clean separation from existing code: algorithms take `&HypergeometricSeries` as input and return new result types. FPS verification as mandatory safety net.

**Critical pitfall:** Coefficient explosion in exact rational polynomial GCD is the #1 performance risk. Paule/Riese documented that 95% of qZeil runtime was in polynomial system solving, reduced to 30-40% only after aggressive content extraction. This optimization must be built into the polynomial module from day one, not retrofitted.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Polynomial Infrastructure** - Build QRatPoly, QRatRationalFunc, polynomial GCD/resultant, and all supporting operations.
   - Addresses: STACK.md (QRatPoly type), ARCHITECTURE.md (poly/ module)
   - Avoids: Pitfall #1 (representation gap), Pitfall #2 (coefficient explosion if content extraction built in)
   - Independent of algorithm knowledge; purely mathematical infrastructure.

2. **q-Gosper's Algorithm** - Implement Gosper normal form (qGFF), q-dispersion via resultant, key equation solver, and antidifference construction.
   - Addresses: FEATURES.md (Features 1-2: term ratio test + q-Gosper)
   - Avoids: Pitfall #3 (qGFF is not standard factorization), Pitfall #7 (q-shift vs ordinary shift)
   - Depends on: Phase 1 (polynomial infrastructure)
   - Independently useful: indefinite q-hypergeometric summation is valuable on its own.

3. **q-Zeilberger + WZ Certificates** - Implement creative telescoping loop, recurrence output, certificate extraction, and WZ verification.
   - Addresses: FEATURES.md (Features 3-4: q-Zeilberger + WZ)
   - Avoids: Pitfall #4 (order search hang -- max order bound), Pitfall #5 (boundary terms), Pitfall #9 (non-terminating input detection)
   - Depends on: Phase 2 (q-Gosper as subroutine)

4. **Python API + Verification** - Expose proving functions via PyO3, add FPS cross-verification, integrate with existing identity database.
   - Addresses: FEATURES.md (Python API), PITFALLS.md (Pitfall #15: API ergonomics)
   - Avoids: Pitfall #11 (confusion with existing eta-quotient prover -- keep separate)
   - Depends on: Phase 3 (core algorithms working)

**Phase ordering rationale:**
- Polynomial infrastructure MUST come first because every algorithm depends on it. Building algorithms without a proper polynomial type leads to incorrect implementations (Pitfall #1).
- q-Gosper before q-Zeilberger because Zeilberger literally calls Gosper as a subroutine. Testing Gosper independently catches algorithm bugs before they compound.
- WZ certificates with Zeilberger (not separate) because certificates are a direct output of the Zeilberger process -- extracting them is a small additional step once Zeilberger works.
- Python API last because it depends on stable Rust interfaces and the API design is informed by what the algorithms actually produce.

**Research flags for phases:**
- Phase 1: Needs careful implementation of subresultant PRS (medium complexity). Reference the algebraics crate's Rust implementation for subresultant patterns.
- Phase 2: Likely needs deeper research on qGFF (q-greatest factorial factorization). Koornwinder 1993 and Paule/Riese 1997 are the authoritative sources. The q-case is simpler than the ordinary case (integer manipulation instead of polynomial root-finding for dispersion).
- Phase 3: Standard patterns from A=B. The creative telescoping loop is conceptually simple but the integration with q-Gosper needs careful parameter passing.
- Phase 4: Standard PyO3 patterns (73 existing functions as template). Low research risk.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Evaluated all Rust polynomial crates; license and capability analysis thorough; build-in-house recommendation is well-justified |
| Features | HIGH | Algorithms well-documented since 1990s; competitor analysis (qZeil, qsum, hipergeo) confirms feature expectations; dependency chain clear |
| Architecture | HIGH | Layer separation (poly -> algorithm -> proof) is standard CAS architecture; integration with existing types verified against codebase |
| Pitfalls | HIGH | Coefficient explosion documented by Paule/Riese with specific numbers (95% -> 30-40%); boundary term errors well-known in WZ literature; representation gap confirmed by codebase inspection |

## Gaps to Address

- **q-Greatest Factorial Factorization (qGFF) implementation details:** The core decomposition step in q-Gosper. PDFs of Koornwinder 1993 and Paule/Riese 1997 were not fully readable during web fetch. The algorithm is described at a high level in multiple corroborating sources (MathWorld, SymPy source, A=B), but the exact q-case implementation (how q-dispersion differs from ordinary dispersion) should be verified against the original papers during Phase 2 planning.

- **Subresultant PRS over QRat coefficient growth:** The algebraics crate (LGPL) has a Rust subresultant implementation that could serve as a reference pattern. The actual coefficient growth behavior for the polynomial sizes arising in q-Gosper (degree 5-30) needs empirical measurement during Phase 1.

- **Non-terminating series handling:** q-Zeilberger is designed for terminating sums. Extending to non-terminating sums (the Chen-Hou-Mu parameter specialization method) is a valuable differentiator but adds complexity. Recommend deferring to a follow-up milestone unless trivially achievable.

- **q-Petkovsek (recurrence solving):** After q-Zeilberger produces a recurrence, solving it for a closed form is the natural next step. This is a separate algorithm (Feature 5 in FEATURES.md) that should be its own phase/milestone.

## Sources

**HIGH CONFIDENCE:**
- Petkovsek, Wilf, Zeilberger, "A=B" (1996) -- foundational reference for all algorithms
- [Koornwinder, "On Zeilberger's algorithm and its q-analogue" (1993)](https://staff.fnwi.uva.nl/t.h.koornwinder/art/1993/zeilbalgo.pdf) -- rigorous q-analogue description
- [Paule & Riese, "A Mathematica q-analogue of Zeilberger's Algorithm" (1997)](https://www3.risc.jku.at/publications/download/risc_117/Paule_Riese.pdf) -- qGFF approach, implementation lessons
- [feanor-math v3.5.13 (MIT)](https://lib.rs/crates/feanor-math) -- evaluated and rejected (slow for rationals)
- [polynomial-ring v0.5.1 (AGPL)](https://lib.rs/crates/polynomial-ring) -- evaluated and rejected (license)
- [Symbolica](https://symbolica.io/license/) -- evaluated and rejected (commercial)
- [SymPy gosper.py source](https://www.aidoczh.com/sympy/_modules/sympy/concrete/gosper.html) -- reference implementation
- q-Kangaroo codebase inspection (QRat, FPS, HypergeometricSeries, linalg) -- integration verified

**MEDIUM CONFIDENCE:**
- [arXiv:2501.03837 (2025)](https://arxiv.org/abs/2501.03837) -- unified reduction for creative telescoping (recent advance)
- [qZeil package](https://www3.risc.jku.at/research/combinat/software/ergosum/RISC/qZeil.html) -- feature set reference
- [algebraics crate GCD](https://docs.rs/algebraics/latest/src/algebraics/polynomial/gcd.rs.html) -- Rust subresultant PRS pattern reference

---
*Research completed: 2026-02-15*
*Ready for roadmap: yes*
