---
status: complete
phase: 32-pdf-reference-manual
source: 32-01-SUMMARY.md, 32-02-SUMMARY.md, 32-03-SUMMARY.md, 32-04-SUMMARY.md, 32-05-SUMMARY.md, 32-06-SUMMARY.md
started: 2026-02-18
updated: 2026-02-18
---

## Current Test

[testing complete]

## Tests

### 1. --help Mentions PDF Manual
expected: Running `q-kangaroo --help` includes a DOCUMENTATION section referencing the PDF manual
result: pass
notes: DOCUMENTATION section says "See the q-Kangaroo Reference Manual (PDF) included with release downloads."

### 2. All 16 Chapter Files Exist
expected: manual/chapters/ contains 16 .typ files (00-title through 15-appendix)
result: pass
notes: All 16 files present and non-empty

### 3. All 81 Functions Documented
expected: 81 func-entry() calls across chapters 05-12
result: pass
notes: 82 total matches (81 in chapters + 1 definition in template.typ). Breakdown: products(7) + partitions(7) + theta(3) + series-analysis(9) + relations(12) + hypergeometric(9) + mock-theta-bailey(27) + identity-proving(7) = 81

### 4. CI build-manual Job Configured
expected: .github/workflows/cli-release.yml contains a build-manual job that compiles Typst to PDF
result: pass
notes: `build-manual:` job with "Build PDF Manual" name, runs on ubuntu-latest

### 5. Worked Examples with Citations
expected: Chapter 13 contains 6 worked examples with scholarly citations
result: pass
notes: 376 lines, 41 lines with citation/author references (Euler, Ramanujan, Jacobi, Rogers, Heine, Watson)

### 6. Maple Migration Table
expected: Chapter 14 contains complete 81-function mapping with aliases
result: pass
notes: 271 lines. Uses Typst table() format (not pipe tables). 17 aliases + 81 functions across 10 groups

### 7. Release Pipeline Wired
expected: create-release job depends on build-manual
result: pass
notes: `needs: [build-linux, build-windows, build-manual]` — all three required before release creation

## Summary

total: 7
passed: 7
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
