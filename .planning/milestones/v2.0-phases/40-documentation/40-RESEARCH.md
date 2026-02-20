# Phase 40: Documentation - Research

**Researched:** 2026-02-20
**Domain:** Typst PDF manual, Rust help system, tab completion, Python API docstrings, Maple migration guide
**Confidence:** HIGH

## Summary

Phase 40 requires updating all documentation surfaces to reflect v2.0 Maple-compatible signatures and new functions. The core finding from this research is that there are **three categories of work**:

1. **Signature replacement** -- Many functions in the manual currently show pre-v2.0 "legacy" signatures (e.g., `sift(series, m, j)`) while the actual Garvan-canonical signatures are different (e.g., `sift(s, q, n, k, T)`). These must be replaced everywhere: manual, help text shows Garvan forms already but examples may need checking.

2. **New function documentation** -- Eight functions added in v2.0 phases 33-39 have help.rs entries but NO manual chapter entries: `lqdegree0`, `checkmult`, `checkprod`, `JAC`, `theta` (general), `jac2prod`, `jac2series`, `qs2jaccombo`. These need full `#func-entry()` blocks in appropriate manual chapters.

3. **Structural documentation** -- The migration guide (chapter 14) needs a complete rewrite to become workflow-oriented with two-column Maple/Kangaroo comparison tables. The function count references throughout the manual say "81 built-in functions" but the actual count is now 89 (help.rs) / 88 (tab completion, which excludes `theta` from the functional count but includes `anames` and `restart`).

**Primary recommendation:** Organize work by documentation surface (manual chapters, migration guide, help.rs, repl.rs tab completion, README, Python docstrings) with manual chapter updates as the heaviest lift.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Replace v1.x signatures entirely with Garvan-canonical forms -- legacy signatures are undocumented (still work, just not shown)
- Full formal math definitions for all new functions: LaTeX-quality summation formulas, product representations, convergence notes -- textbook-reference level
- New functions slot into existing categories by domain (theta under Products, checkmult under Analysis, etc.) -- no separate v2.0 section
- Worked examples cross-reference Garvan's actual Maple worksheets where applicable (e.g., "As in qseries Example 3.2") to help researchers map between systems
- Workflow-oriented migration guide organization: sections like "Computing eta products", "Finding congruences", "Theta identities" -- task-oriented, not alphabetical
- Two-column table format for migration guide: left column Maple code, right column Kangaroo code -- compact and scannable
- Migration guide lives as a new chapter in the PDF manual (Typst) -- single authoritative source, ships with the binary
- Focus only on remaining differences in migration guide -- where translation is still needed. Don't enumerate identical syntax
- Show Garvan-only signatures in help() -- no legacy forms shown, consistent with manual approach
- 1 example per function in help -- help is a quick reference, the manual has depth
- help(partition_count) silently redirects to numbpart -- no deprecation notice, no mention of rename
- Add new "Jacobi Products" help category for jac2prod, jac2series, qs2jaccombo; other new functions slot into existing categories

### Claude's Discretion
- Python API docstring update scope (targeted updates vs broader overhaul)
- Tab completion additions (straightforward -- just add new function names)
- Exact wording and tone of help entries
- Migration guide chapter placement within the manual

### Deferred Ideas (OUT OF SCOPE)
(None specified)
</user_constraints>

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Typst | 0.14.2 | PDF manual compilation | Already used; `typst compile manual/main.typ` |
| in-dexter | 0.7.2 | Back-of-book index generation | Already imported in template.typ |
| Rust doc comments | N/A | Help system in help.rs | Compiled into the binary |
| rustyline | 17.0 | Tab completion in repl.rs | Already used for REPL |
| PyO3 doc comments | N/A | Python API docstrings | Already used in dsl.rs |

### Supporting
| Tool | Purpose | When to Use |
|------|---------|-------------|
| `#func-entry()` macro | Typst template for function reference entries | All new function docs in manual chapters |
| `#repl()` / `#repl-block()` | REPL transcript formatting in Typst | All manual examples |
| `#index[]` / `#index-main[]` | Index entries in Typst | New functions need index entries |
| `#table()` | Typst table macro | Migration guide two-column tables |

## Architecture Patterns

### Manual Chapter Structure
```
manual/
  main.typ                      # Master include list (no changes needed)
  template.typ                  # func-entry, repl, repl-block (no changes needed)
  chapters/
    05-products.typ             # UPDATE: aqprod/etaq/jacprod/etc Garvan sigs + ADD JAC, theta, jac2prod, jac2series, qs2jaccombo
    06-partitions.typ           # UPDATE: partition_count -> numbpart, add numbpart(n,m)
    07-theta.typ                # No new functions (theta2/3/4 unchanged)
    08-series-analysis.typ      # UPDATE: sift Garvan sig + ADD lqdegree0, checkmult, checkprod
    09-relations.typ            # UPDATE: findprod Garvan sig, findcong Garvan sig
    14-maple-migration.typ      # REWRITE: workflow-oriented with two-column tables
    01-quick-start.typ          # UPDATE: partition_count(100) -> numbpart(100), function count
    04-expression-language.typ  # UPDATE: function count, partition_count -> numbpart in list
    00-title.typ                # UPDATE: function count
    03-cli-usage.typ            # UPDATE: function count
```

### Help System Structure (help.rs)
```
general_help()  -- Grouped function listing (already has "Jacobi Products:" category)
FUNC_HELP[]     -- Array of 89 FuncHelp structs (already up to date for v2.0)
function_help() -- Lookup by name with partition_count -> numbpart redirect
```

### Tab Completion Structure (repl.rs)
```
canonical_function_names() -- Vec of 88 names (already includes v2.0 functions)
```

### Key Finding: Help System is ALREADY Updated
The help.rs file already shows Garvan-canonical signatures for all 89 functions. The general_help() listing already has the "Jacobi Products:" category with JAC, jac2prod, jac2series, qs2jaccombo. The help entries use Garvan `(s, q, n, k, T)` style for sift, `(FL, T, M, Q)` for findprod, `(QS, T)` for findcong, etc.

### Key Finding: Tab Completion is ALREADY Updated
The repl.rs canonical_function_names() already includes all v2.0 additions: `lqdegree0` is missing though -- let me verify...

Actually, looking more carefully at repl.rs line 64: the Analysis group in tab completion is:
```
"sift", "qdegree", "lqdegree", "qfactor",
"prodmake", "etamake", "jacprodmake", "mprodmake", "qetamake",
```
This is MISSING: `lqdegree0`, `checkmult`, `checkprod`. These need to be added.

And checking ALL_FUNCTION_NAMES in eval.rs (line 3751):
```
"sift", "qdegree", "lqdegree", "lqdegree0", "qfactor",
"checkmult", "checkprod",
```
So eval.rs has them but repl.rs tab completion does NOT. This is a bug to fix.

### Pattern: func-entry Template

Every new function in the manual uses the `#func-entry()` Typst macro:

```typst
#func-entry(
  name: "function_name",
  signature: "function_name(arg1, arg2, ...)",
  description: [
    Text description with $math$ notation.
    #index[primary index entry]
  ],
  math-def: [
    $ formal_definition $
  ],
  params: (
    ([arg1], [Type], [description]),
    ([arg2], [Type], [description]),
  ),
  examples: (
    ("input_expression", "output_text"),
  ),
  edge-cases: (
    [Edge case 1.],
    [Edge case 2.],
  ),
  related: ("func1", "func2"),
)
```

### Pattern: Migration Guide Two-Column Table

```typst
#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`maple_code`], [`kangaroo_code`],
  ...
)
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Function reference layout | Custom Typst formatting | `#func-entry()` macro in template.typ | Consistent with all 81+ existing entries |
| REPL examples in manual | Raw code blocks | `#repl()` and `#repl-block()` helpers | Adds `q>` prompt styling automatically |
| Index entries | Manual index management | `#index[]` and `#index-main[]` from in-dexter | Automatic back-of-book index generation |

## Common Pitfalls

### Pitfall 1: Signature Mismatch Between Help and Manual
**What goes wrong:** The manual shows one signature, help shows another, creating user confusion.
**Why it happens:** Help.rs was updated in v2.0 phases but manual chapters were not.
**How to avoid:** For every function, verify the manual signature matches help.rs exactly.
**Warning signs:** Manual says `sift(series, m, j)` but help says `sift(s, q, n, k, T)`.

### Pitfall 2: Function Count References
**What goes wrong:** Multiple places say "81 built-in functions" but we now have 89.
**Why it happens:** The count was correct pre-v2.0 and was never updated.
**How to avoid:** Search all .typ files for "81" and update to the correct count. The correct number: 89 unique function help entries. But the general_help header says "q-Kangaroo Functions" without a count, so the issue is only in manual chapters.
**Locations to fix:**
- `00-title.typ` line 21: "all 81 built-in functions"
- `01-quick-start.typ` line 84: "all 81 built-in functions"
- `03-cli-usage.typ` line 129: "all 81 functions"
- `04-expression-language.typ` line 130: "81 built-in functions organized into 8 groups" (now need to say ~10 groups or adjust)

### Pitfall 3: partition_count vs numbpart in Manual
**What goes wrong:** The manual still calls the function `partition_count` everywhere, but the canonical name is now `numbpart`.
**Why it happens:** v2.0 Phase 34 reversed the direction -- numbpart became canonical, partition_count became alias.
**How to avoid:** Replace `partition_count` with `numbpart` in all manual chapters. The function entry in chapter 06 needs renaming. The migration guide's alias table has the direction backwards.
**Specific locations:**
- `06-partitions.typ`: name: "partition_count" -> "numbpart", signature, examples
- `01-quick-start.typ`: `partition_count(100)` -> `numbpart(100)`
- `04-expression-language.typ`: `partition_count` in function list
- `14-maple-migration.typ`: alias table row is backwards, complete mapping shows wrong direction

### Pitfall 4: Migration Guide Has Wrong Alias Direction
**What goes wrong:** Chapter 14 says `numbpart` is a Maple alias for `partition_count`, but the reverse is now true.
**Why it happens:** Pre-v2.0, partition_count was canonical. v2.0 Phase 34 swapped them.
**How to avoid:** The migration guide is being rewritten anyway, but ensure the alias table reflects: `partition_count` is an alias for `numbpart` (or better: just show `numbpart` as the function name directly since it's Garvan-compatible).

### Pitfall 5: Tab Completion Missing New Functions
**What goes wrong:** User types `check<TAB>` and gets nothing.
**Why it happens:** repl.rs canonical_function_names() was not updated when checkmult/checkprod/lqdegree0 were added to eval.rs.
**How to avoid:** Cross-reference eval.rs ALL_FUNCTION_NAMES against repl.rs canonical_function_names() and add missing entries.
**Missing from tab completion:** `lqdegree0`, `checkmult`, `checkprod`

### Pitfall 6: Manual Signatures Must Show Garvan Form, Not Legacy
**What goes wrong:** Researcher reads manual, types legacy signature, gets confused when it doesn't match Garvan's documentation.
**Decision:** Per CONTEXT.md, replace v1.x signatures entirely with Garvan-canonical forms.
**Key changes needed:**

| Function | Manual Currently Shows | Should Show (Garvan) |
|----------|----------------------|---------------------|
| sift | `sift(series, m, j)` | `sift(s, q, n, k, T)` |
| prodmake | `prodmake(series, max_n)` | `prodmake(f, q, T)` |
| etamake | `etamake(series, max_n)` | `etamake(f, q, T)` |
| jacprodmake | `jacprodmake(series, max_n)` | `jacprodmake(f, q, T)` or `jacprodmake(f, q, T, P)` |
| mprodmake | `mprodmake(series, max_n)` | `mprodmake(f, q, T)` |
| qetamake | `qetamake(series, max_n)` | `qetamake(f, q, T)` |
| qfactor | `qfactor(series)` | `qfactor(f, q)` or `qfactor(f, q, T)` |
| aqprod | `aqprod(coeff_num, coeff_den, power, n_or_infinity, order)` | `aqprod(a, q, n)` or `aqprod(a, q, infinity, T)` |
| qbin | `qbin(n, k, order)` | `qbin(q, m, n)` |
| etaq | `etaq(b, t, order)` | `etaq(q, delta, T)` or `etaq(q, [deltas], T)` |
| jacprod | `jacprod(a, b, order)` | `jacprod(a, b, q, T)` |
| tripleprod | `tripleprod(coeff_num, coeff_den, power, order)` | `tripleprod(z, q, T)` |
| quinprod | `quinprod(coeff_num, coeff_den, power, order)` | `quinprod(z, q, T)` |
| winquist | `winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)` | `winquist(a, b, q, T)` |
| findprod | `findprod([series], max_coeff, max_exp)` | `findprod(FL, T, M, Q)` |
| findcong | `findcong(series, [moduli])` | `findcong(QS, T)` or `findcong(QS, T, LM)` or `findcong(QS, T, LM, XSET)` |
| findlincombo | `findlincombo(target, [candidates], topshift)` | `findlincombo(f, L, SL, q, topshift)` |
| findhomcombo | `findhomcombo(target, [candidates], degree, topshift)` | `findhomcombo(f, L, q, n, topshift)` |
| findnonhomcombo | `findnonhomcombo(target, [candidates], degree, topshift)` | `findnonhomcombo(f, L, q, n, topshift)` |
| findlincombomodp | `findlincombomodp(target, [candidates], p, topshift)` | `findlincombomodp(f, L, SL, p, q, topshift)` |
| findhomcombomodp | `findhomcombomodp(target, [candidates], p, degree, topshift)` | `findhomcombomodp(f, L, p, q, n, topshift)` |
| findhom | `findhom([series], degree, topshift)` | `findhom(L, q, n, topshift)` |
| findnonhom | `findnonhom([series], degree, topshift)` | `findnonhom(L, q, n, topshift)` |
| findhommodp | `findhommodp([series], p, degree, topshift)` | `findhommodp(L, p, q, n, topshift)` |
| findmaxind | `findmaxind([series], topshift)` | `findmaxind(L, T)` |
| findpoly | `findpoly(x, y, deg_x, deg_y, topshift)` | `findpoly(x, y, q, dx, dy)` or `findpoly(x, y, q, dx, dy, check)` |

## Code Examples

### Example 1: New func-entry for checkmult (Series Analysis chapter)

```typst
#func-entry(
  name: "checkmult",
  signature: "checkmult(QS, T) or checkmult(QS, T, 'yes')",
  description: [
    Test if the coefficients of a q-series are multiplicative, i.e.,
    $f(m n) = f(m) f(n)$ for all coprime $m, n$. Checks all coprime pairs
    with $2 <= m, n <= T\/2$ and $m n <= T$. Prints MULTIPLICATIVE or
    NOT MULTIPLICATIVE and returns 1 or 0.
    #index[multiplicative]
    #index[checkmult]
  ],
  math-def: [
    A sequence $\{a_n\}$ is *multiplicative* if $a_1 = 1$ and
    $a_(m n) = a_m dot a_n$ whenever $gcd(m, n) = 1$.

    Many important number-theoretic functions (Euler's $phi$, Ramanujan's
    $tau$, divisor sums $sigma_k$) are multiplicative.
  ],
  params: (
    ([QS], [Series], [The input q-series to test]),
    ([T], [Integer], [Maximum index to check coprime pairs up to]),
    (['yes'], [String (optional)], [If provided, prints ALL failing pairs instead of stopping at first]),
  ),
  examples: (
    ("f := partition_gf(50)\ncheckmult(f, 30)",
     "NOT MULTIPLICATIVE at (2, 3)\n0"),
  ),
  edge-cases: (
    [Returns 1 if multiplicative, 0 if not.],
    [With the optional `'yes'` argument, prints all failing pairs instead of stopping at the first failure.],
    [The test is exhaustive up to the given bound $T$, but does not constitute a proof for all $n$.],
  ),
  related: ("checkprod", "findcong", "sift"),
)
```

### Example 2: Updated sift signature (Garvan form)

```typst
#func-entry(
  name: "sift",
  signature: "sift(s, q, n, k, T)",
  description: [
    Extract the arithmetic subsequence of coefficients from a power series.
    Given a series $f(q) = sum a_i q^i$, the call `sift(f, q, n, k, T)` returns
    a new series whose $i$-th coefficient is the $(n i + k)$-th coefficient
    of the input, using terms up to $q^T$.
    #index[sift]
    #index[arithmetic subsequence]
  ],
  ...
)
```

### Example 3: Migration Guide Workflow Section

```typst
== Computing Eta Products

Maple and q-Kangaroo use identical syntax for eta products:

#table(
  columns: (1fr, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Maple*], [*q-Kangaroo*],
  ),
  [`etaq(q, 1, 20)`], [`etaq(q, 1, 20)`],
  [`etaq(q, [1,2,3], 20)`], [`etaq(q, [1,2,3], 20)`],
  [`prodmake(f, q, 10)`], [`prodmake(f, q, 10)`],
  [`etamake(f, q, 10)`], [`etamake(f, q, 10)`],
)

No translation needed -- syntax is identical.
```

## State of the Art

| Old Approach (pre-v2.0) | Current Approach (v2.0) | When Changed | Impact on Docs |
|--------------------------|------------------------|--------------|----------------|
| `partition_count` canonical | `numbpart` canonical | Phase 34 | Manual chapter 06 and all references need updating |
| Legacy int-triple signatures | Garvan monomial signatures | Phases 33-39 | All product/analysis/relation signatures need updating |
| 81 functions | 89 functions | Phases 33-39 | Count references in 4+ manual locations |
| No Jacobi product algebra | JAC/jac2prod/jac2series/qs2jaccombo | Phase 39 | Need new chapter section or entries in Products |
| No checkmult/checkprod | Both available | Phase 38 | Need new entries in Series Analysis chapter |
| No general theta | theta(z, q, T) | Phase 39 | Need new entry in Products or Theta chapter |
| No lqdegree0 | lqdegree0(f) alias | Phase 36 | Need new entry in Series Analysis chapter |
| Migration guide: alphabetical alias table | Workflow-oriented sections | Phase 40 (this phase) | Complete rewrite of chapter 14 |

## Detailed Inventory of Changes Needed

### Manual Chapter Updates

**Chapter 00 (Title):** Update "81 built-in functions" to correct count.

**Chapter 01 (Quick Start):**
- `partition_count(100)` -> `numbpart(100)` (line 38)
- `help aqprod` example: update signature shown to Garvan form if different
- "81 built-in functions" -> correct count (line 84)
- "all 81 built-in functions" -> correct count

**Chapter 03 (CLI Usage):** "81 functions" reference (line 129).

**Chapter 04 (Expression Language):**
- "81 built-in functions organized into 8 groups" -> update count and group count
- `partition_count` in Partitions list -> `numbpart`
- Function groups list needs updating to include Jacobi Products category
- Group counts need updating (Analysis now has 12 not 9, etc.)

**Chapter 05 (Products):**
- All 7 function signatures -> Garvan forms
- aqprod: `aqprod(a, q, n)` or `aqprod(a, q, infinity, T)`
- qbin: `qbin(q, m, n)`
- etaq: `etaq(q, delta, T)` or `etaq(q, [deltas], T)`
- jacprod: `jacprod(a, b, q, T)`
- tripleprod: `tripleprod(z, q, T)`
- quinprod: `quinprod(z, q, T)`
- winquist: `winquist(a, b, q, T)`
- ADD: JAC, theta (general), jac2prod, jac2series, qs2jaccombo entries
- All examples need updating to use Garvan calling conventions
- Parameter tables need rewriting for new signatures

**Chapter 06 (Partitions):**
- `partition_count` -> `numbpart` everywhere
- Add `numbpart(n, m)` variant documentation
- Related function references: `partition_count` -> `numbpart`

**Chapter 07 (Theta):** Minimal changes -- theta2/3/4 signatures unchanged.

**Chapter 08 (Series Analysis):**
- sift: `sift(series, m, j)` -> `sift(s, q, n, k, T)`
- All *make functions: update to Garvan `(f, q, T)` form
- qfactor: `qfactor(series)` -> `qfactor(f, q)` or `qfactor(f, q, T)`
- ADD: `lqdegree0`, `checkmult`, `checkprod` entries
- All examples need updating

**Chapter 09 (Relations):**
- ALL 12 function signatures -> Garvan forms
- findprod: `findprod([series], max_coeff, max_exp)` -> `findprod(FL, T, M, Q)`
- findcong: `findcong(series, [moduli])` -> `findcong(QS, T)` or `findcong(QS, T, LM)` etc.
- All findXXX functions need signature updates
- Examples need updating

**Chapter 14 (Migration Guide):** Complete rewrite per CONTEXT.md decisions.

### Help System (help.rs)
Already up to date with Garvan signatures. No changes needed for content.
BUT: the help.rs comment says "89 functions" while general_help() does not mention a count. Verify no action needed.

### Tab Completion (repl.rs)
**Missing from canonical_function_names():** `lqdegree0`, `checkmult`, `checkprod`
The count test expects 88 but should be 91 after adding these 3 (or verify actual count).
Wait -- let me recount: eval.rs ALL_FUNCTION_NAMES has `lqdegree0`, `checkmult`, `checkprod` (line 3751-3752). repl.rs does NOT have them (line 64-65). This is a confirmed bug. These 3 names need to be added and the count test updated.

### Python API (dsl.rs)
The Python API uses its own signatures (taking QSession as first arg). These are NOT affected by the Maple compatibility changes -- the Python API was always its own interface. However:
- `partition_count` function name in Python should stay as-is (it's the Python function name, not a REPL alias)
- Check if any docstrings reference old function names or outdated examples
- The scope here is targeted: verify docstrings are accurate, not a broader overhaul

### README.md
- Quick Start section uses `partition_count(n)` in the Python example -- this is fine since the Python API function IS called `partition_count`
- The "81 functions" reference in the Documentation section (line 56) needs updating
- "all 81 functions" should become the correct count

## Open Questions

1. **Exact function count for the manual**
   - What we know: help.rs has 89 FuncHelp entries. ALL_FUNCTION_NAMES has ~88 entries. Tab completion has 88 entries (but is missing 3). The manual pre-v2.0 said "81".
   - What's unclear: Should we count session functions (anames, restart, read) in the "built-in functions" count? The help entry count (89) includes JAC+theta+jac2prod+jac2series+qs2jaccombo but the general_help also lists them separately.
   - Recommendation: Count all functions visible in `help` output. The general_help lists 89 functions by name. Use "89" as the count in the manual. Alternatively, just say "nearly 90 built-in functions" to avoid constant updates.

2. **Products chapter structure for Jacobi product functions**
   - What we know: CONTEXT says new functions slot into existing categories by domain. Products chapter 05 covers aqprod, qbin, etaq, jacprod, tripleprod, quinprod, winquist.
   - What's unclear: Should JAC/theta/jac2prod/jac2series/qs2jaccombo go at the end of chapter 05, or should there be a subsection? The help system has a separate "Jacobi Products" category.
   - Recommendation: Add a new `== Jacobi Product Algebra` subsection at the end of chapter 05 (Products), containing JAC, jac2prod, jac2series, qs2jaccombo. Place `theta` (general) in chapter 07 (Theta Functions).

3. **Python API docstring scope**
   - What we know: Python API uses its own calling conventions (QSession first arg, different param names). The function is still called `partition_count` in Python.
   - What's unclear: How much Python docstring updating is needed?
   - Recommendation: Targeted review only. Check that examples in docstrings still work and that any references to function names match reality. No need for a broader overhaul since the Python API was not changed by v2.0.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-cli/src/help.rs` -- Current help text with all 89 Garvan-canonical entries (read directly)
- `crates/qsym-cli/src/eval.rs` -- Current dispatch with Garvan+legacy dual signatures, ALL_FUNCTION_NAMES (read directly)
- `crates/qsym-cli/src/repl.rs` -- Current tab completion list, 88 entries (read directly)
- `manual/chapters/*.typ` -- All 15 manual chapters examined for current state (read directly)
- `manual/template.typ` -- func-entry, repl, repl-block templates (read directly)
- `crates/qsym-python/src/dsl.rs` -- Python API function signatures and docstrings (read directly)
- `README.md` -- Current README with Quick Start examples (read directly)

### Secondary (MEDIUM confidence)
- MEMORY.md project memory -- milestone history, function counts, architecture notes

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all tools already in use, no new dependencies
- Architecture: HIGH -- examined every file that needs changes
- Pitfalls: HIGH -- identified all signature mismatches by cross-referencing help.rs vs manual chapters
- New function gaps: HIGH -- confirmed 8 functions in help.rs with no manual entries

**Research date:** 2026-02-20
**Valid until:** Stable -- documentation phase, no external dependency changes
