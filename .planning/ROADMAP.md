# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (shipped 2026-02-16)
- v1.4 Installation & Build Guide - Phases 22-23 (shipped 2026-02-17)
- v1.5 Interactive REPL - Phases 24-28 (in progress)

## Phases

<details>
<summary>v1.0 Core Engine (Phases 1-8) - SHIPPED 2026-02-14</summary>

- [x] Phase 1: Expression Foundation (3/3 plans) -- 2026-02-13
- [x] Phase 2: Simplification & Series Engine (3/3 plans) -- 2026-02-13
- [x] Phase 3: Core q-Series & Partitions (4/4 plans) -- 2026-02-13
- [x] Phase 4: Series Analysis (7/7 plans) -- 2026-02-13
- [x] Phase 5: Python API (4/4 plans) -- 2026-02-13
- [x] Phase 6: Hypergeometric Series (4/4 plans) -- 2026-02-14
- [x] Phase 7: Identity Proving (4/4 plans) -- 2026-02-14
- [x] Phase 8: Mock Theta & Bailey Chains (4/4 plans) -- 2026-02-14

See `.planning/milestones/v1.0-MILESTONE-AUDIT.md` for details.

</details>

<details>
<summary>v1.1 Polish & Publish (Phases 9-12) - SHIPPED 2026-02-15</summary>

- [x] Phase 9: Package Rename & Structure (2/2 plans) -- 2026-02-14
- [x] Phase 10: PyPI Packaging & Metadata (2/2 plans) -- 2026-02-14
- [x] Phase 11: CI/CD Pipeline (2/2 plans) -- 2026-02-15
- [x] Phase 12: Documentation & UX Polish (4/4 plans) -- 2026-02-15

See `.planning/milestones/v1.1-ROADMAP.md` for details.

</details>

<details>
<summary>v1.2 Algorithmic Identity Proving (Phases 13-17) - SHIPPED 2026-02-16</summary>

- [x] Phase 13: Polynomial Infrastructure (3/3 plans) -- 2026-02-15
- [x] Phase 14: q-Gosper Algorithm (3/3 plans) -- 2026-02-16
- [x] Phase 15: q-Zeilberger & WZ Certificates (3/3 plans) -- 2026-02-16
- [x] Phase 16: Extensions (3/3 plans) -- 2026-02-16
- [x] Phase 17: Python API & Documentation (2/2 plans) -- 2026-02-16

See `.planning/milestones/v1.2-ROADMAP.md` for details.

</details>

<details>
<summary>v1.3 Documentation & Vignettes (Phases 18-21) - SHIPPED 2026-02-16</summary>

- [x] Phase 18: Docstring Enrichment (4/4 plans) -- 2026-02-16
- [x] Phase 19: Vignette Expansion (3/3 plans) -- 2026-02-16
- [x] Phase 20: New Vignettes & Migration Guide (3/3 plans) -- 2026-02-16
- [x] Phase 21: Sphinx Site Polish (2/2 plans) -- 2026-02-16

See `.planning/milestones/v1.3-ROADMAP.md` for details.

</details>

<details>
<summary>v1.4 Installation & Build Guide (Phases 22-23) - SHIPPED 2026-02-17</summary>

- [x] Phase 22: Installation Documentation (2/2 plans) -- 2026-02-17
- [x] Phase 23: Verification & Cross-References (2/2 plans) -- 2026-02-17

See `.planning/milestones/v1.4-ROADMAP.md` for details.

</details>

### v1.5 Interactive REPL (In Progress)

**Milestone Goal:** A standalone Rust executable providing an interactive REPL with Maple-style syntax, all 79 q-Kangaroo functions, variable assignment, LaTeX output, and save-to-file -- giving researchers a terminal-based Maple replacement for q-series computation.

- [x] **Phase 24: Parser & AST** - Maple-style expression parser with function calls, assignment, arithmetic, and literals
- [x] **Phase 25: Evaluator & Function Dispatch** - AST evaluator connecting all 79 functions to qsym-core with variable environment and text output
- [x] **Phase 26: REPL Shell & Session** - Interactive line-editing shell with history, tab completion, help system, and session configuration
- [x] **Phase 27: Output Commands & Polish** - LaTeX rendering and file save commands for computed results
- [x] **Phase 28: Binary Packaging** - Standalone executables for Windows and Linux with CI integration

## Phase Details

### Phase 24: Parser & AST
**Goal**: Users can type Maple-style expressions and the system correctly parses them into an internal representation
**Depends on**: Nothing (first phase of v1.5; depends only on existing qsym-core)
**Requirements**: PARSE-01, PARSE-02, PARSE-03, PARSE-04
**Success Criteria** (what must be TRUE):
  1. Typing `aqprod(q,q,infinity,20)` parses into a function-call AST node with 4 arguments including the `infinity` keyword
  2. Typing `f := etaq(1,1,20)` parses into an assignment AST node with variable name `f` and a function-call value
  3. Typing `f + g`, `f - g`, `f * g`, `-f`, and `3*f` all parse into correct arithmetic AST nodes
  4. Integer literals (`50`), rational literals (`3/4`), and the `infinity` keyword parse into their respective AST leaf nodes
**Plans:** 2 plans

Plans:
- [x] 24-01-PLAN.md -- Crate scaffold and AST/Token/Error type definitions
- [x] 24-02-PLAN.md -- Lexer and Pratt parser with comprehensive tests

### Phase 25: Evaluator & Function Dispatch
**Goal**: Users can call any of the 79 q-Kangaroo functions by name and see computed series output in the terminal
**Depends on**: Phase 24
**Requirements**: FUNC-01, FUNC-02, FUNC-03, FUNC-04, FUNC-05, FUNC-06, FUNC-07, FUNC-08, SESS-01, OUT-01
**Success Criteria** (what must be TRUE):
  1. User types `aqprod(q,q,infinity,20)` and sees the Euler function series printed as human-readable text (e.g., `1 - q - q^2 + q^5 + ... + O(q^20)`)
  2. User types `f := partition_gf(30)` then `f` on the next line and sees the partition generating function -- variables persist across lines
  3. All 8 function groups work: q-Pochhammer/products, partitions, theta, series analysis, relation discovery, hypergeometric, mock theta/Bailey, and identity proving
  4. User types `f := etaq(1,1,20); g := etaq(2,1,20); f * g` and sees the product of two eta quotients
  5. Calling a function with wrong argument count or type prints a descriptive error message
**Plans:** 3 plans

Plans:
- [x] 25-01-PLAN.md -- Evaluator core, variable environment, parser [...] extension, and text output formatting
- [x] 25-02-PLAN.md -- Function dispatch groups 1-4 (q-Pochhammer, partitions, theta, series analysis)
- [x] 25-03-PLAN.md -- Function dispatch groups 5-8 (relations, hypergeometric, mock theta/Bailey, identity proving)

### Phase 26: REPL Shell & Session
**Goal**: Users have a polished interactive terminal experience with line editing, history, help, and session control
**Depends on**: Phase 25
**Requirements**: REPL-01, REPL-02, REPL-03, REPL-04, SESS-02, SESS-03
**Success Criteria** (what must be TRUE):
  1. User launches the executable and gets a prompt; up/down arrows recall previous commands; history persists after restart
  2. User types `aq` then presses Tab and sees completion candidates (aqprod); typing `f` then Tab completes to defined variable names
  3. User types `help` and sees a list of available functions; `help aqprod` shows the signature and a one-line description
  4. User types a malformed expression and sees a descriptive parse error without the session crashing
  5. User types `set precision 50` to change default truncation order; `clear` resets variables; `quit` exits cleanly
**Plans:** 2 plans

Plans:
- [x] 26-01-PLAN.md -- Rustyline REPL loop with history, session commands, multi-line input, and error recovery
- [x] 26-02-PLAN.md -- Tab completion (functions, commands, variables) and help system (8 categories, 81 functions)

### Phase 27: Output Commands & Polish
**Goal**: Users can extract computed results as LaTeX or save session work to files
**Depends on**: Phase 26
**Requirements**: OUT-02, OUT-03
**Success Criteria** (what must be TRUE):
  1. User types `latex` after a computation and sees LaTeX source for the last result; `latex f` outputs LaTeX for variable `f`
  2. User types `save results.txt` and the session transcript or last result is written to a file on disk
**Plans**: TBD

Plans:
- [ ] 27-01: LaTeX output command and save-to-file command

### Phase 28: Binary Packaging
**Goal**: Researchers can download and run a single executable on Windows or Linux without installing Rust
**Depends on**: Phase 27
**Requirements**: BIN-01, BIN-02
**Success Criteria** (what must be TRUE):
  1. `cargo build --release` in `crates/qsym-cli/` produces a standalone `.exe` for Windows (x86_64-pc-windows-gnu) that runs without a Rust toolchain
  2. CI builds produce a Linux binary (x86_64-unknown-linux-gnu) that runs on a fresh Linux machine
  3. The binary starts, shows a welcome banner, and enters the REPL -- all 79 functions callable
**Plans**: TBD

Plans:
- [ ] 28-01: Release build configuration and CI workflow for binary artifacts

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Expression Foundation | v1.0 | 3/3 | Complete | 2026-02-13 |
| 2. Simplification & Series Engine | v1.0 | 3/3 | Complete | 2026-02-13 |
| 3. Core q-Series & Partitions | v1.0 | 4/4 | Complete | 2026-02-13 |
| 4. Series Analysis | v1.0 | 7/7 | Complete | 2026-02-13 |
| 5. Python API | v1.0 | 4/4 | Complete | 2026-02-13 |
| 6. Hypergeometric Series | v1.0 | 4/4 | Complete | 2026-02-14 |
| 7. Identity Proving | v1.0 | 4/4 | Complete | 2026-02-14 |
| 8. Mock Theta & Bailey Chains | v1.0 | 4/4 | Complete | 2026-02-14 |
| 9. Package Rename & Structure | v1.1 | 2/2 | Complete | 2026-02-14 |
| 10. PyPI Packaging & Metadata | v1.1 | 2/2 | Complete | 2026-02-14 |
| 11. CI/CD Pipeline | v1.1 | 2/2 | Complete | 2026-02-15 |
| 12. Documentation & UX Polish | v1.1 | 4/4 | Complete | 2026-02-15 |
| 13. Polynomial Infrastructure | v1.2 | 3/3 | Complete | 2026-02-15 |
| 14. q-Gosper Algorithm | v1.2 | 3/3 | Complete | 2026-02-16 |
| 15. q-Zeilberger & WZ Certificates | v1.2 | 3/3 | Complete | 2026-02-16 |
| 16. Extensions | v1.2 | 3/3 | Complete | 2026-02-16 |
| 17. Python API & Documentation | v1.2 | 2/2 | Complete | 2026-02-16 |
| 18. Docstring Enrichment | v1.3 | 4/4 | Complete | 2026-02-16 |
| 19. Vignette Expansion | v1.3 | 3/3 | Complete | 2026-02-16 |
| 20. New Vignettes & Migration Guide | v1.3 | 3/3 | Complete | 2026-02-16 |
| 21. Sphinx Site Polish | v1.3 | 2/2 | Complete | 2026-02-16 |
| 22. Installation Documentation | v1.4 | 2/2 | Complete | 2026-02-17 |
| 23. Verification & Cross-References | v1.4 | 2/2 | Complete | 2026-02-17 |
| 24. Parser & AST | v1.5 | 2/2 | Complete | 2026-02-17 |
| 25. Evaluator & Function Dispatch | v1.5 | 3/3 | Complete | 2026-02-17 |
| 26. REPL Shell & Session | v1.5 | 2/2 | Complete | 2026-02-17 |
| 27. Output Commands & Polish | v1.5 | 1/1 | Complete | 2026-02-18 |
| 28. Binary Packaging | v1.5 | 1/1 | Complete | 2026-02-18 |
