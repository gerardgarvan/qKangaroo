# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (shipped 2026-02-16)
- v1.4 Installation & Build Guide - Phases 22-23 (shipped 2026-02-17)
- v1.5 Interactive REPL - Phases 24-28 (shipped 2026-02-18)
- v1.6 CLI Hardening & Manual - Phases 29-32 (in progress)

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

<details>
<summary>v1.5 Interactive REPL (Phases 24-28) - SHIPPED 2026-02-18</summary>

- [x] Phase 24: Parser & AST (2/2 plans) -- 2026-02-17
- [x] Phase 25: Evaluator & Function Dispatch (3/3 plans) -- 2026-02-18
- [x] Phase 26: REPL Shell & Session (2/2 plans) -- 2026-02-18
- [x] Phase 27: Output Commands & Polish (1/1 plan) -- 2026-02-18
- [x] Phase 28: Binary Packaging (1/1 plan) -- 2026-02-18

See `.planning/milestones/v1.5-ROADMAP.md` for details.

</details>

### v1.6 CLI Hardening & Manual (In Progress)

**Milestone Goal:** Make q-Kangaroo a fully self-contained, production-quality command-line tool with zero DLL dependencies, script execution, robust error handling, and a comprehensive PDF reference manual.

- [x] **Phase 29: Static Linking** - Self-contained binary with zero DLL dependencies -- 2026-02-18
- [x] **Phase 30: Script Execution & CLI Flags** - Non-interactive batch mode and argument handling -- 2026-02-18
- [x] **Phase 31: Error Hardening & Exit Codes** - Robust diagnostics and machine-readable exit status -- 2026-02-18
- [ ] **Phase 32: PDF Reference Manual** - Comprehensive typeset documentation for all 81 functions

## Phase Details

### Phase 29: Static Linking
**Goal**: Users download a single executable file with zero external dependencies
**Depends on**: Nothing (first phase of v1.6; changes build infrastructure only)
**Requirements**: BUILD-01, BUILD-02, BUILD-03
**Success Criteria** (what must be TRUE):
  1. Running `q-kangaroo --version` works on a clean Windows machine with no DLLs in the directory or PATH
  2. The GitHub release archive for Windows contains exactly one file: the .exe (no DLL files)
  3. CI builds from bundled GMP/MPFR/MPC source without requiring pre-installed system libraries
**Plans**: 2 plans

Plans:
- [x] 29-01-PLAN.md -- Enable static GMP/MPFR/MPC linking and verify locally
- [x] 29-02-PLAN.md -- Rewrite CI workflow for static builds and single-file archives

### Phase 30: Script Execution & CLI Flags
**Goal**: Users can run q-Kangaroo non-interactively via script files, piped input, or command-line expressions
**Depends on**: Phase 29 (builds against static-linked binary infrastructure)
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, EXEC-01, EXEC-02, EXEC-03, EXEC-04, EXEC-05, EXEC-06
**Success Criteria** (what must be TRUE):
  1. User can run `q-kangaroo script.qk` to execute a file containing `#` comments and multi-line statements, then the process exits
  2. User can run `echo "1+1" | q-kangaroo` and see the result with no banner or prompt
  3. User can run `q-kangaroo -c "qpoch(a,q,5)"` to evaluate a single expression and exit
  4. User can run `q-kangaroo --help` to see a usage summary listing all flags and file argument syntax
  5. User can run `read("file.qk")` in the REPL to execute a script file within the current session
**Plans**: 3 plans

Plans:
- [x] 30-01-PLAN.md -- Extend lexer/parser/AST with comments, newlines, strings; create script.rs execution engine
- [x] 30-02-PLAN.md -- Refactor main.rs with CLI argument parsing, mode dispatch, and read() function
- [x] 30-03-PLAN.md -- Subprocess integration tests for all CLI modes and flags

### Phase 31: Error Hardening & Exit Codes
**Goal**: Users get clear, actionable error messages and scripts/tools can rely on distinct exit codes for every failure mode
**Depends on**: Phase 30 (script mode and CLI flags must exist for exit codes and error context to apply)
**Requirements**: EXIT-01, EXIT-02, EXIT-03, EXIT-04, EXIT-05, EXIT-06, EXIT-07, ERR-01, ERR-02, ERR-03, ERR-04, ERR-05
**Success Criteria** (what must be TRUE):
  1. Running a script with a typo on line 5 produces an error showing `script.qk:5:` with a human-readable message, then exits with code 1
  2. Running `q-kangaroo nonexistent.qk` prints "file not found" with the OS error message and exits with code 66
  3. Running `q-kangaroo --bogus` prints a clear "unknown flag" message with `--help` suggestion and exits with code 2
  4. A script that triggers a qsym-core panic displays a translated human-readable message (not a Rust panic backtrace) and exits with code 70
  5. In the REPL, errors print a message but the session continues; in scripts, the first error stops execution
**Plans**: 2 plans

Plans:
- [x] 31-01-PLAN.md -- Error infrastructure: ScriptResult variants, multiline error rendering, panic translation, REPL --help
- [x] 31-02-PLAN.md -- Integration tests for all 12 requirement IDs (EXIT-01 through EXIT-07, ERR-01 through ERR-05)

### Phase 32: PDF Reference Manual
**Goal**: Users have a comprehensive, professionally typeset PDF reference manual covering all 81 functions
**Depends on**: Phase 31 (documents final CLI flags, script mode, error messages, and exit codes)
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05, DOC-06
**Success Criteria** (what must be TRUE):
  1. A PDF file exists in the GitHub release archive alongside the binary, covering all 81 functions with mathematical definitions
  2. The manual includes a CLI usage section documenting all flags, script execution, exit codes, and error messages
  3. The manual includes worked examples and a Maple migration quick-reference
  4. Running `q-kangaroo --help` mentions the PDF manual by name
**Plans**: 6 plans

Plans:
- [ ] 32-01-PLAN.md -- Typst project infrastructure, templates, and introductory chapters (00-04)
- [ ] 32-02-PLAN.md -- Function reference: Products, Partitions, Theta, Series Analysis (26 functions)
- [ ] 32-03-PLAN.md -- Function reference: Relations, Hypergeometric (21 functions)
- [ ] 32-04-PLAN.md -- Function reference: Mock Theta/Bailey, Identity Proving (34 functions)
- [ ] 32-05-PLAN.md -- Worked Examples, Maple Migration, and Index chapters
- [ ] 32-06-PLAN.md -- CI workflow for PDF compilation and --help update

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
| 25. Evaluator & Function Dispatch | v1.5 | 3/3 | Complete | 2026-02-18 |
| 26. REPL Shell & Session | v1.5 | 2/2 | Complete | 2026-02-18 |
| 27. Output Commands & Polish | v1.5 | 1/1 | Complete | 2026-02-18 |
| 28. Binary Packaging | v1.5 | 1/1 | Complete | 2026-02-18 |
| 29. Static Linking | v1.6 | 2/2 | Complete | 2026-02-18 |
| 30. Script Execution & CLI Flags | v1.6 | 3/3 | Complete | 2026-02-18 |
| 31. Error Hardening & Exit Codes | v1.6 | 2/2 | Complete | 2026-02-18 |
| 32. PDF Reference Manual | v1.6 | 0/6 | Not started | - |
