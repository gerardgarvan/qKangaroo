# Milestones

## v1.0: Core Engine (Complete)

**Shipped:** 2026-02-14
**Phases:** 1-8 (32 plans, 578 Rust tests, 9 Python integration tests)

### What Shipped

| Phase | Capability |
|-------|-----------|
| 1 | Expression foundation — hash-consed arena, arbitrary precision, LaTeX/Unicode rendering |
| 2 | FPS engine, simplification engine, infinite product generators |
| 3 | q-Pochhammer, q-binomials, named products (eta/Jacobi/triple/quintuple/Winquist), theta functions, partitions, rank/crank |
| 4 | prodmake (Andrews' algorithm), qfactor, sift, etamake/jacprodmake/mprodmake/qetamake, relation discovery (12+ functions) |
| 5 | Python API via PyO3 — 38 DSL functions, QSession/QExpr/QSeries, batch generation |
| 6 | Hypergeometric series — eval_phi/psi, 6 summation formulas, Heine/Sears/Watson/Bailey transforms |
| 7 | Identity proving — eta quotients, cusps, valence formula, TOML identity database |
| 8 | Mock theta (20 functions), Appell-Lerch sums, Bailey pairs/lemma/chains/discovery |

### Key Metrics

- 73 Python DSL functions across 10 groups
- Full Garvan `qseries` parity plus extensions
- UAT: 47/47 tests passed across all 8 phases (1 issue found and fixed)

### Key Decisions

See STATE.md Accumulated Context for full decision log.
