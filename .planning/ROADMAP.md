# Roadmap: q-Kangaroo

## Milestones

- v1.0 Core Engine - Phases 1-8 (shipped 2026-02-14)
- v1.1 Polish & Publish - Phases 9-12 (shipped 2026-02-15)
- v1.2 Algorithmic Identity Proving - Phases 13-17 (shipped 2026-02-16)
- v1.3 Documentation & Vignettes - Phases 18-21 (shipped 2026-02-16)
- v1.4 Installation & Build Guide - Phases 22-23

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

### v1.4 Installation & Build Guide

**Milestone Goal:** Bulletproof installation instructions for both end users (pip install) and contributors (build from source), with a verification script to confirm the setup works.

#### Phase 22: Installation Documentation
**Goal**: Users have complete, self-contained installation instructions for every supported platform and workflow
**Depends on**: Nothing (documentation-only, no code dependencies)
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05
**Success Criteria** (what must be TRUE):
  1. A user on any supported platform can follow INSTALL.md from scratch to a working `import q_kangaroo` without consulting any other document
  2. A contributor on Cygwin/Windows can follow the build-from-source section to compile the Rust crate and run `maturin develop` successfully
  3. A contributor on Linux can follow the build-from-source section to compile and install from source successfully
  4. A user encountering a common failure (GMP not found, wrong Rust target, DLL loading error) finds the solution in the troubleshooting section
  5. The Sphinx-rendered installation.rst page contains the same detail as INSTALL.md, properly formatted with RST directives
**Plans:** 2 plans

Plans:
- [x] 22-01-PLAN.md -- INSTALL.md with pip install, build-from-source (Linux + Cygwin/Windows), and troubleshooting
- [x] 22-02-PLAN.md -- installation.rst Sphinx page mirroring INSTALL.md content

#### Phase 23: Verification & Cross-References
**Goal**: Users can verify their installation works and discover the installation guide from every entry point
**Depends on**: Phase 22 (cross-references point to content from Phase 22; verification script validates the installation path)
**Requirements**: VRFY-01, VRFY-02, XREF-01, XREF-02
**Success Criteria** (what must be TRUE):
  1. Running `python check_install.py` prints pass/fail for Python version, import, GMP loading, and basic computation
  2. Running `python check_install.py --dev` additionally checks Rust, cargo, maturin, GMP headers, and C compiler availability
  3. README.md contains a clear pointer to INSTALL.md for anyone looking for build instructions
  4. The Sphinx landing page (index.rst) links to the new installation guide so docs readers find it immediately
**Plans:** 2 plans

Plans:
- [ ] 23-01-PLAN.md -- check_install.py with end-user and --dev verification
- [ ] 23-02-PLAN.md -- README.md and index.rst cross-reference updates

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
| 23. Verification & Cross-References | v1.4 | 0/2 | Not started | - |
