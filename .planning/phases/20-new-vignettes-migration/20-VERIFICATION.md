---
phase: 20-new-vignettes-migration
verified: 2026-02-17T03:00:00Z
status: gaps_found
score: 12/14 must-haves verified
re_verification: false
gaps:
  - truth: "maple_migration.ipynb shows at least 30 common operations with side-by-side code"
    status: partial
    reason: "Multiple pre-computed outputs are mathematically incorrect"
    artifacts:
      - path: "docs/examples/maple_migration.ipynb"
        issue: "6 wrong pre-computed outputs in cells 4, 7, 19, 21, 22"
    missing:
      - "Fix cell-4 aqprod output and label"
      - "Fix cell-7 jacprod output"
      - "Fix cell-19 qfactor output format"
      - "Fix cell-21 findlincombo output"
      - "Fix cell-22 findmaxind output"
  - truth: "series_analysis.ipynb shows findlincombo, findhom, and findpoly"
    status: partial
    reason: "findhom described in markdown but not demonstrated with a code cell"
    artifacts:
      - path: "docs/examples/series_analysis.ipynb"
        issue: "findhom has no code cell demonstration"
    missing:
      - "Add a code cell demonstrating findhom"
---

# Phase 20: New Vignettes & Migration Guide Verification Report

**Phase Goal:** Create 4 new notebooks covering gaps: newcomer onboarding, series analysis workflow, identity proving workflow, and Maple migration.
**Verified:** 2026-02-17T03:00:00Z
**Status:** gaps_found
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | getting_started.ipynb walks newcomer from import to identity verification | VERIFIED | 22 cells: imports -> QSession -> aqprod -> partition_gf -> etaq/jacprod -> findcong -> prove_eta_id |
| 2  | getting_started.ipynb shows QSession, aqprod, partition_gf, findcong | VERIFIED | Cells 3, 5, 11, 17 demonstrate all four with correct outputs |
| 3  | getting_started.ipynb demonstrates etaq and jacprod | VERIFIED | Cell 14; outputs match math (Euler function, theta4) |
| 4  | getting_started.ipynb ends with prove_eta_id | VERIFIED | Cell 20 calls prove_eta_id, output status=proved |
| 5  | series_analysis.ipynb demonstrates prodmake -> etamake -> sift pipeline | VERIFIED | Cells 4, 7, 10, 13, 16-17 with correct outputs |
| 6  | series_analysis.ipynb shows findlincombo, findhom, findpoly | PARTIAL | findlincombo (cell 23) and findpoly (cell 26) in code; findhom only in markdown |
| 7  | series_analysis.ipynb demonstrates findcong | VERIFIED | Cell 29 with correct Ramanujan congruences |
| 8  | series_analysis.ipynb covers jacprodmake and mprodmake | VERIFIED | Cell 10 (jacprodmake), cell 13 (mprodmake) |
| 9  | identity_proving.ipynb demonstrates q-Zeilberger | VERIFIED | Cell 7 with correct output (order 1) |
| 10 | identity_proving.ipynb shows WZ certificate verification | VERIFIED | Cell 12 verify_wz verified=True; cell 14 shows certificate |
| 11 | identity_proving.ipynb demonstrates q-Petkovsek | VERIFIED | Cell 16 full pipeline with closed form |
| 12 | identity_proving.ipynb shows prove_nonterminating | VERIFIED | Cell 20 proves identity; cell 22 rejects wrong RHS |
| 13 | identity_proving.ipynb demonstrates find_transformation_chain | VERIFIED | Cell 25 finds Heine 1 chain; cell 27 no-chain case |
| 14 | identity_proving.ipynb shows q-Gosper | VERIFIED | Cells 3-4: summable and non-summable examples |
| 15 | maple_migration.ipynb covers all 13 function groups | VERIFIED | Groups 1-13 all present with tables and code |
| 16 | maple_migration.ipynb shows 30+ operations with side-by-side code | PARTIAL | 34+ operations demonstrated, BUT 6 pre-computed outputs are wrong |
| 17 | index.rst lists all 9 notebooks | VERIFIED | 3 toctree sections, all 9 listed |

**Score:** 12/14 plan must-haves verified (getting_started: 4/4, series_analysis: 3/4, identity_proving: 6/6, maple_migration+index: 3/4)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| docs/examples/getting_started.ipynb | Newcomer tutorial | VERIFIED | 22 cells, valid JSON, nbformat 4, all outputs present |
| docs/examples/series_analysis.ipynb | Analysis workflow | VERIFIED | 32 cells, valid JSON, nbformat 4, all outputs present |
| docs/examples/identity_proving.ipynb | Identity proving | VERIFIED | 31 cells, valid JSON, nbformat 4, all outputs present |
| docs/examples/maple_migration.ipynb | Migration guide | ISSUES | 48 cells, valid JSON, but 6 wrong outputs |
| docs/examples/index.rst | Gallery with 9 notebooks | VERIFIED | 3 toctree sections, audience hints |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| getting_started.ipynb | q_kangaroo API | imports | WIRED | All verified against __init__.py |
| series_analysis.ipynb | q_kangaroo API | imports | WIRED | All verified; findhom not called |
| identity_proving.ipynb | q_kangaroo API | imports | WIRED | All 6 algorithm functions verified |
| maple_migration.ipynb | q_kangaroo API | import * | WIRED | All called functions in __all__ |
| index.rst | *.ipynb files | toctree | WIRED | All 9 notebooks exist on disk |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
|-------------|--------|----------------|
| DOC-09: Newcomer onboarding | SATISFIED | None |
| DOC-10: Series analysis workflow | MOSTLY SATISFIED | findhom not demonstrated with code |
| DOC-11: Identity proving workflow | SATISFIED | None |
| DOC-12: Maple migration guide | PARTIALLY SATISFIED | 6 wrong pre-computed outputs |
| DOC-13: All function groups covered | SATISFIED | All 13 groups present |

### Anti-Patterns Found

| File | Cell | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| maple_migration.ipynb | cell-4 | Wrong output | BLOCKER | aqprod(s,1,1,1,3,10) shows 1-q-q^2+q^3 (wrong); correct: 1-q-q^2+q^4+q^5-q^6+O(q^10) |
| maple_migration.ipynb | cell-4 | Wrong label | WARNING | Label says (1;q)_3 but code computes (q;q)_3 |
| maple_migration.ipynb | cell-7 | Wrong output | BLOCKER | jacprod(1,2) shows 1-q-q^3+q^6+...; correct: 1-2q+2q^4-2q^9+... (per Rust test) |
| maple_migration.ipynb | cell-19 | Wrong format | WARNING | qfactor prints only inner dict, missing scalar/is_exact keys |
| maple_migration.ipynb | cell-21 | Wrong output | WARNING | findlincombo returns [0] not [Fraction(1,1)] |
| maple_migration.ipynb | cell-22 | Wrong output | WARNING | findmaxind returns [0] not [0,1] |

### Human Verification Required

#### 1. Pre-computed Output Accuracy
**Test:** Run all notebook code cells against live q-kangaroo.
**Expected:** Outputs match engine behavior.
**Why human:** No Python runtime available on this system.

#### 2. Notebook Rendering
**Test:** Open notebooks in Jupyter Lab, check LaTeX and table rendering.
**Expected:** All equations and tables render correctly.
**Why human:** Requires browser/Jupyter instance.

#### 3. Sphinx Build
**Test:** Run sphinx-build and verify all 9 notebooks in built site.
**Expected:** All toctree entries resolve.
**Why human:** Requires Python environment with Sphinx.

### Gaps Summary

**Gap 1 (BLOCKER): maple_migration.ipynb has 6 wrong pre-computed outputs.**

The aqprod output (cell-4) shows the expansion of a 2-factor product instead of the correct 3-factor (q;q)_3 product. The getting_started notebook correctly shows the output as 1 - q - q^2 + q^4 + q^5 - q^6 + O(q^10) for the same function call.

The jacprod output (cell-7) is wrong. The Rust test jacprod_1_2_is_theta4 confirms J(1,2) = theta4 = 1 - 2q + 2q^4 - 2q^9 + 2q^16 + ..., but maple_migration shows 1 - q - q^3 + q^6 + q^10 - q^15 + O(q^20).

The findlincombo output (cell-21) shows [0] for Euler theorem (distinct=odd) but should show [Fraction(1,1)]. The series_analysis notebook correctly shows [Fraction(1, 1)] for the same relation (cell-23).

The findmaxind output (cell-22) shows [0] (1 independent) for 3 series where 2 are independent.

Root cause: Plan 20-03 executor fabricated outputs rather than using a Rust test harness.

**Gap 2 (MINOR): series_analysis.ipynb missing findhom code demo.**

The must-have requires findlincombo, findhom, AND findpoly to be shown. findhom is described in markdown but has no code cell. Adding one findhom code cell would close this gap.

---

_Verified: 2026-02-17T03:00:00Z_
_Verifier: Claude (gsd-verifier)_
