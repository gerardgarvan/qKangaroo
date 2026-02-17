# Requirements: v1.5 Interactive REPL

**Defined:** 2026-02-17
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

**Goal:** A standalone Rust executable providing an interactive REPL with Maple-style syntax, all 79 q-Kangaroo functions, variable assignment, LaTeX output, and save-to-file -- giving researchers a terminal-based Maple replacement for q-series computation.

## Parser

- [ ] **PARSE-01**: Parser handles Maple-style function calls with positional arguments (aqprod(q,q,infinity,20), partition_count(50))
- [ ] **PARSE-02**: Parser handles variable assignment (:=) and variable references (f := etaq(1,1,20); f)
- [ ] **PARSE-03**: Parser handles arithmetic on series: +, -, *, unary negation, and integer scalar multiplication
- [ ] **PARSE-04**: Parser handles infinity keyword, integer literals, and rational literals (3/4)

## REPL

- [ ] **REPL-01**: Interactive REPL with line editing, up/down history navigation, and persistent history file
- [ ] **REPL-02**: Tab completion for function names and variable names
- [ ] **REPL-03**: Help system (help, help function_name) with function signatures and descriptions
- [ ] **REPL-04**: Parse/runtime errors print descriptive messages without crashing the session

## Functions

- [ ] **FUNC-01**: q-Pochhammer and product functions (aqprod, etaq, jacprod, tripleprod, quinprod, winquist) with Maple-compatible names
- [ ] **FUNC-02**: Partition functions (partition_count, partition_gf, rank_gf, crank_gf, restricted partitions)
- [ ] **FUNC-03**: Theta functions (theta2, theta3, theta4)
- [ ] **FUNC-04**: Series analysis functions (prodmake, etamake, jacprodmake, mprodmake, sift, qfactor, qetamake)
- [ ] **FUNC-05**: Relation discovery functions (findlincombo, findhom, findpoly, findcong, findnonhom, findmaxind, findprod, and modp variants)
- [ ] **FUNC-06**: Hypergeometric functions (phi, psi, try_summation, heine1/2/3, sears_transform, watson_transform)
- [ ] **FUNC-07**: Mock theta (20 functions), Appell-Lerch (bilateral, g2, g3), and Bailey chain functions
- [ ] **FUNC-08**: Identity proving and algorithmic functions (q_gosper, q_zeilberger, verify_wz, q_petkovsek, prove_nonterminating, find_transformation_chain)

## Output

- [ ] **OUT-01**: Default text output showing series in human-readable format (1 - q - q^2 + q^5 + O(q^20))
- [ ] **OUT-02**: latex command outputs LaTeX for the last result or a named variable
- [ ] **OUT-03**: save filename writes results or session transcript to a file

## Session

- [ ] **SESS-01**: Variables persist across lines within a session (f := ...; g := ...; f*g)
- [ ] **SESS-02**: Configurable truncation order (set precision N changes default series terms)
- [ ] **SESS-03**: clear resets variables/session, quit/exit exits cleanly

## Binary

- [ ] **BIN-01**: Compiles to standalone executable for Windows (x86_64-pc-windows-gnu)
- [ ] **BIN-02**: Compiles to standalone executable for Linux (x86_64-unknown-linux-gnu)

## Out of Scope

| Feature | Reason |
|---------|--------|
| GUI / graphical interface | Terminal REPL is the goal; GUI is a separate project |
| Script file execution | Can be added later; interactive-first for v1.5 |
| Networking / package management | Standalone tool, no external dependencies at runtime |
| macOS binary | No macOS CI; can cross-compile later |
| Static GMP linking | Separate build system change; bundle DLL for now |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PARSE-01 | Phase 24 | Complete |
| PARSE-02 | Phase 24 | Complete |
| PARSE-03 | Phase 24 | Complete |
| PARSE-04 | Phase 24 | Complete |
| REPL-01 | Phase 26 | Pending |
| REPL-02 | Phase 26 | Pending |
| REPL-03 | Phase 26 | Pending |
| REPL-04 | Phase 26 | Pending |
| FUNC-01 | Phase 25 | Pending |
| FUNC-02 | Phase 25 | Pending |
| FUNC-03 | Phase 25 | Pending |
| FUNC-04 | Phase 25 | Pending |
| FUNC-05 | Phase 25 | Pending |
| FUNC-06 | Phase 25 | Pending |
| FUNC-07 | Phase 25 | Pending |
| FUNC-08 | Phase 25 | Pending |
| OUT-01 | Phase 25 | Pending |
| OUT-02 | Phase 27 | Pending |
| OUT-03 | Phase 27 | Pending |
| SESS-01 | Phase 25 | Pending |
| SESS-02 | Phase 26 | Pending |
| SESS-03 | Phase 26 | Pending |
| BIN-01 | Phase 28 | Pending |
| BIN-02 | Phase 28 | Pending |

**Coverage:**
- v1.5 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-02-17*
