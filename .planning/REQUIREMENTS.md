# Requirements: v1.4 Installation & Build Guide

**Defined:** 2026-02-17
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

**Goal:** Bulletproof installation instructions for both end users (pip install) and contributors (build from source) on Cygwin/Windows and Linux, with a verification script to confirm the setup works.

## Installation Documentation

- [ ] **INST-01**: INSTALL.md at repo root with complete end-user pip install instructions (Python version, wheel availability, GMP bundling, verification command)
- [ ] **INST-02**: INSTALL.md build-from-source section with step-by-step Cygwin/Windows instructions (Cygwin packages, Rust toolchain, MinGW GCC, GMP setup, Python venv, maturin develop, PATH configuration)
- [ ] **INST-03**: INSTALL.md build-from-source section with step-by-step Linux instructions (apt/yum packages, Rust, GMP-dev, maturin)
- [ ] **INST-04**: INSTALL.md troubleshooting section covering common failures (GMP not found, wrong Rust target, PATH issues, DLL loading errors)
- [ ] **INST-05**: docs/installation.rst rewritten to mirror INSTALL.md content (full Sphinx-rendered version with the same detail)

## Verification Tooling

- [ ] **VRFY-01**: check_install.py script that verifies Python version, import q_kangaroo, GMP loading, basic computation (partition_count), and prints pass/fail for each check
- [ ] **VRFY-02**: check_install.py also verifies build-from-source prerequisites when run with `--dev` flag (Rust, cargo, maturin, GMP headers, C compiler)

## Cross-references

- [ ] **XREF-01**: README.md updated to point to INSTALL.md for detailed build instructions
- [ ] **XREF-02**: Sphinx landing page (index.rst) updated to reference new installation guide

## Out of Scope

| Feature | Reason |
|---------|--------|
| macOS build instructions | macOS CI deferred; no test environment available |
| Docker/container setup | Overkill for a Python library; pip install is sufficient |
| conda-forge recipe | Separate infrastructure work for a future milestone |
| Static GMP linking | Separate build system change, not documentation |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| INST-01 | Phase 22 | Pending |
| INST-02 | Phase 22 | Pending |
| INST-03 | Phase 22 | Pending |
| INST-04 | Phase 22 | Pending |
| INST-05 | Phase 22 | Pending |
| VRFY-01 | Phase 23 | Pending |
| VRFY-02 | Phase 23 | Pending |
| XREF-01 | Phase 23 | Pending |
| XREF-02 | Phase 23 | Pending |

**Coverage:**
- v1.4 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0

---
*Requirements defined: 2026-02-17*
