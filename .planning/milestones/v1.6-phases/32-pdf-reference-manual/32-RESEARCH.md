# Phase 32: PDF Reference Manual - Research

**Researched:** 2026-02-18
**Domain:** Typst typesetting, mathematical documentation, PDF generation, CI/CD
**Confidence:** HIGH

## Summary

This phase produces a comprehensive, professionally typeset PDF reference manual for q-Kangaroo's 81 functions, CLI usage, expression language, worked examples, and Maple migration table. The user has locked Typst as the typesetting tool. The document follows a hybrid organization: introductory chapters followed by an alphabetical/grouped function reference, with a back-of-book index cross-referencing function names, math concepts, and Maple function names.

Typst (current stable: v0.14.2, released 2025-12-12) is a modern, Rust-based typesetting system with native math support, multi-file `#include` organization, fast compilation (milliseconds), and straightforward GitHub Actions integration via `typst-community/setup-typst@v4`. Its math mode natively supports product/summation notation (`product_(k=0)^n`, `sum_(k=0)^n`), fractions (`frac(a,b)`), infinity (`oo`), and custom operators (`op("name")`), which covers all q-series mathematical notation needs. The `in-dexter` package (v0.7.2) provides back-of-book index generation with sub-entries and page consolidation.

**Primary recommendation:** Use a multi-file Typst project split by chapter, with `in-dexter` for index generation, `typst-community/setup-typst@v4` for CI, and function reference grouped by domain (matching the existing 8 groups in `help.rs`) rather than flat A-Z, since the domain grouping provides mathematical context that researchers need.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Document structure & flow
- Hybrid organization: introductory chapters followed by an alphabetical function reference
- Intro chapters: Quick Start tutorial (2-3 pages: install, first expression, first script), CLI usage summary (flags/modes, brief exit code mention), expression language reference (variables, operators, types)
- Function reference section: Claude's discretion on whether to group by domain or flat A-Z -- pick whichever makes 81 functions most navigable
- Comprehensive back-of-book index: function names, math concepts, and Maple function names cross-referenced
- Target length: 100+ pages (comprehensive treatment)
- CLI chapter: summary-level coverage of exit codes and error messages (not a sysadmin reference)

#### Function entry depth
- Full treatment per function: signature, mathematical definition, parameter descriptions, 2-3 examples, edge cases, related functions
- Mathematical notation: formal product/sum notation for core q-series functions (qpoch, etaq, jacprod, etc.); informal prose descriptions for utility/helper functions
- Maple equivalents: NOT in individual function entries -- all Maple mappings go in the migration quick-reference table only
- Examples: REPL transcript style showing `q>` input and actual output
- Edge cases: document parameter constraints, invalid input behavior, and known limitations

#### Typesetting tool
- Typst (not LaTeX) -- modern, fast, single binary, Rust-based
- CI builds PDF on release via GitHub Actions (install Typst, compile from source)
- Page layout and file organization: Claude's discretion

#### Worked examples & migration
- Both identity verification examples AND research workflow examples
- Number of examples: Claude's discretion based on function families and document balance
- Cite source papers for identities (e.g., "This proves Theorem 2.1 from Andrews 1986")
- Maple migration: side-by-side mapping table only (no prose translation guide)

### Claude's Discretion
- Function reference grouping (by domain vs flat A-Z)
- Page layout (single column vs mixed)
- Typst file organization (single file vs split by chapter)
- Number and selection of worked examples
- Overall tone and style (blend of Maple help pages and mathematical reference)
- Font choices and visual design

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope

</user_constraints>

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Typst | 0.14.2 | PDF typesetting from markup source | User decision; Rust-based, fast, modern math support |
| in-dexter | 0.7.2 | Back-of-book index generation | Only mature Typst index package; supports sub-entries, page consolidation |
| typst-community/setup-typst | v4 | GitHub Actions Typst installer | Official community action; handles caching, version pinning |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| softprops/action-gh-release | v2 | Attach PDF to GitHub release | Already used in cli-release.yml |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| in-dexter | Manual state/query index | Much more code, no sub-entries, reinventing the wheel |
| Multi-file Typst | Single monolithic .typ | Single file works but unwieldy at 100+ pages; multi-file enables parallel editing |

## Architecture Patterns

### Recommended File Organization
```
manual/
  main.typ                    # Master document: metadata, set rules, includes
  template.typ                # Shared styling: fonts, colors, page layout, function-entry template
  chapters/
    01-quick-start.typ        # Quick Start tutorial (2-3 pages)
    02-installation.typ       # Installation (CLI binary, from source)
    03-cli-usage.typ          # CLI flags, modes, exit codes, session commands
    04-expression-language.typ # Variables, operators, types, q keyword, infinity, %
    05-products.typ           # 7 functions: aqprod, qbin, etaq, jacprod, tripleprod, quinprod, winquist
    06-partitions.typ         # 7 functions: partition_count through crank_gf
    07-theta.typ              # 3 functions: theta2, theta3, theta4
    08-series-analysis.typ    # 9 functions: sift through qetamake
    09-relations.typ          # 12 functions: findlincombo through findpoly
    10-hypergeometric.typ     # 9 functions: phi, psi, try_summation, heine1-3, sears, watson, chain
    11-mock-theta-bailey.typ  # 27 functions: mock theta (20), appell-lerch (3), bailey (4)
    12-identity-proving.typ   # 7 functions: prove_eta_id through prove_nonterminating
    13-worked-examples.typ    # Identity verification + research workflow examples
    14-maple-migration.typ    # Side-by-side mapping table
    15-index.typ              # Back-of-book index via in-dexter
```

### Pattern 1: Function Entry Template
**What:** A reusable Typst function that renders each of the 81 function entries consistently.
**When to use:** Every function in chapters 05-12.
**Example:**
```typst
// template.typ
#let func-entry(
  name: none,
  signature: none,
  description: none,
  math-def: none,
  params: (),
  examples: (),
  edge-cases: (),
  related: (),
) = {
  heading(level: 3, name)
  // Index the function name
  index(name)

  // Signature in monospace
  block(fill: luma(245), inset: 8pt, radius: 4pt, width: 100%)[
    #raw(signature)
  ]

  // Description
  description

  // Mathematical definition (if provided)
  if math-def != none {
    [*Mathematical definition:*]
    math-def
  }

  // Parameters table
  if params.len() > 0 {
    [*Parameters:*]
    table(
      columns: (auto, auto, auto),
      [*Name*], [*Type*], [*Description*],
      ..params.flatten()
    )
  }

  // Examples in REPL transcript style
  if examples.len() > 0 {
    [*Examples:*]
    for ex in examples {
      block(fill: luma(248), inset: 8pt, radius: 4pt, width: 100%)[
        #raw(ex, lang: none)
      ]
    }
  }

  // Edge cases
  if edge-cases.len() > 0 {
    [*Edge cases and constraints:*]
    for ec in edge-cases {
      [- #ec]
    }
  }

  // Related functions
  if related.len() > 0 {
    [*Related:* #related.join(", ")]
  }
}
```

### Pattern 2: REPL Transcript Styling
**What:** Consistent styling for all code examples showing `q>` prompt, input, and output.
**When to use:** All 81 function examples plus worked examples chapter.
**Example:**
```typst
#let repl-example(input, output) = {
  block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
    #set text(font: "DejaVu Sans Mono", size: 9pt)
    #text(fill: rgb("#666666"))[q> ]#raw(input)\
    #raw(output)
  ]
}
```

### Pattern 3: Mathematical Definition Blocks
**What:** Formal product/sum notation for core q-series functions; informal prose for utilities.
**When to use:** Core functions (products, partitions, theta, hypergeometric, mock theta) get formal math. Utility functions (sift, qdegree, findlincombo) get prose descriptions.
**Example:**
```typst
// Formal definition for aqprod
$ (a; q)_n = product_(k=0)^(n-1) (1 - a q^k) $

// Informal for sift
// "Extracts the arithmetic subsequence: returns a new series whose
//  n-th coefficient is the (m*n + j)-th coefficient of the input."
```

### Pattern 4: Multi-file #include Structure
**What:** Master document sets global rules, then includes chapter files in order.
**When to use:** main.typ is the only compiled file.
**Example:**
```typst
// main.typ
#import "@preview/in-dexter:0.7.2": *
#import "template.typ": *

#set document(title: "q-Kangaroo Reference Manual", author: "q-Kangaroo Contributors")
#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "New Computer Modern", size: 11pt)
#set heading(numbering: "1.1")
#set math.equation(numbering: "(1)")

// Title page
#include "chapters/00-title.typ"

// Table of contents
#outline(depth: 3, indent: auto)
#pagebreak()

// Chapters
#include "chapters/01-quick-start.typ"
#include "chapters/02-installation.typ"
// ... etc

// Back-of-book index
#pagebreak()
#make-index(title: [Index])
```

### Anti-Patterns to Avoid
- **Maple names in function entries:** Per user decision, Maple equivalents go ONLY in the migration table (chapter 14), never in individual function entries.
- **Single monolithic Typst file:** At 100+ pages, a single file becomes unmanageable. Use multi-file #include.
- **LaTeX-style verbosity:** Typst's syntax is cleaner; avoid over-engineering with excessive `#set` rules. Let Typst defaults handle most formatting.
- **Mixing Python API and CLI signatures:** The PDF documents the CLI REPL interface. Function signatures should match the CLI form (`aqprod(coeff_num, coeff_den, power, n_or_infinity, order)`) not the Python API form (`aqprod(session, num, den, pow, n, order)`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Back-of-book index | Custom state/query index system | `in-dexter` package v0.7.2 | Handles page consolidation, sub-entries, alphabetical sections automatically |
| Table of contents | Manual TOC | `#outline()` built-in | Automatic, depth-configurable, styled via show rules |
| Equation numbering | Manual numbering | `#set math.equation(numbering: "(1)")` | Automatic cross-file numbering |
| PDF metadata | Raw PDF manipulation | `#set document(title: ..., author: ...)` | Typst handles PDF metadata natively |
| GitHub Actions Typst install | Manual curl/untar | `typst-community/setup-typst@v4` | Handles caching, version pinning, cross-platform |

**Key insight:** Typst has excellent built-in support for TOC, equation numbering, cross-references, and PDF metadata. The only external dependency needed is `in-dexter` for the back-of-book index.

## Common Pitfalls

### Pitfall 1: Semicolons in Math Mode
**What goes wrong:** Typst math mode interprets `;` as an array separator. Writing `$(a;q)_n$` may produce unexpected layout.
**Why it happens:** Semicolons in Typst math create array arguments for matrices/vectors.
**How to avoid:** Test the `(a; q)_n` notation early. If semicolons cause issues, use `(a thin q)_n` with a thin space or define a custom command. Based on initial research, `$(a; q)_n$` appears to work correctly in Typst -- the semicolon only triggers array behavior inside function calls, not in bare math expressions.
**Warning signs:** Unexpected spacing or layout in Pochhammer symbol notation.

### Pitfall 2: #include vs #import Confusion
**What goes wrong:** Using `#import` when `#include` is needed, or vice versa.
**Why it happens:** `#import` brings names into scope (for functions/variables). `#include` splices content into the document flow.
**How to avoid:** Use `#include` for chapter content files. Use `#import` only for `template.typ` and external packages (`in-dexter`).
**Warning signs:** Missing content or "unknown function" errors.

### Pitfall 3: Inconsistent Function Signatures
**What goes wrong:** Documenting 81 functions with inconsistent parameter naming or ordering.
**Why it happens:** Each function has different arity and conventions. Without a template, drift is inevitable.
**How to avoid:** Use the func-entry template for ALL 81 functions. Extract signatures from the authoritative source: `help.rs` and `eval.rs` in `crates/qsym-cli/src/`.
**Warning signs:** Parameter names in the manual don't match `help <func>` output.

### Pitfall 4: Missing Functions in the Index
**What goes wrong:** Some functions or Maple names aren't indexed, making the manual less useful for lookup.
**Why it happens:** Manual index entry insertion is easy to forget across 81 functions.
**How to avoid:** The func-entry template should automatically call `#index[name]` for every function. Additionally, index Maple aliases in the migration chapter.
**Warning signs:** A function name search in the PDF index returns no results.

### Pitfall 5: CLI vs Python API Confusion
**What goes wrong:** Examples show Python API calls instead of CLI REPL syntax.
**Why it happens:** Existing docs (Sphinx) document the Python API. The PDF documents the CLI.
**How to avoid:** All examples use `q>` prompt style. Function signatures match CLI form (no session parameter). Cross-check against `help.rs` entries.
**Warning signs:** `s = QSession()` or `from q_kangaroo import` appearing in examples.

### Pitfall 6: CI Workflow Not Including PDF in Release Archives
**What goes wrong:** PDF is compiled but not attached to the GitHub release.
**Why it happens:** The existing `cli-release.yml` packages binaries only. PDF must be explicitly added.
**How to avoid:** Add a Typst compile job to the release workflow, then include the PDF in the release artifacts alongside the binary archives.
**Warning signs:** Release artifacts contain only `.tar.gz`/`.zip` binary archives, no PDF.

## Code Examples

### Typst Math: q-Pochhammer Symbol
```typst
// Display equation with label
$ (a; q)_n = product_(k=0)^(n-1) (1 - a q^k) $ <eq:qpoch-finite>

// Infinite product
$ (a; q)_oo = product_(k=0)^oo (1 - a q^k) $ <eq:qpoch-infinite>
```

### Typst Math: Theta Functions
```typst
$ theta_3 (q) = sum_(n = -oo)^oo q^(n^2) = 1 + 2 sum_(n=1)^oo q^(n^2) $
```

### Typst Math: Basic Hypergeometric Series
```typst
$ attach(, tl: r) phi_s mat(a_1, a_2, ..., a_r; b_1, b_2, ..., b_s; q, z)
  = sum_(n=0)^oo frac(
    (a_1; q)_n (a_2; q)_n dots.c (a_r; q)_n,
    (b_1; q)_n (b_2; q)_n dots.c (b_s; q)_n (q; q)_n
  ) [(-1)^n q^(binom(n,2))]^(1+s-r) z^n $
```

### Typst Math: q-Binomial
```typst
$ binom(n, k)_q = frac((q;q)_n, (q;q)_k (q;q)_(n-k)) $
```

### Typst Math: Dedekind Eta
```typst
$ eta(tau) = q^(1\/24) product_(k=1)^oo (1 - q^k), quad q = e^(2 pi i tau) $
```

### REPL Transcript Example Block
```typst
#let repl(input, output) = block(
  fill: luma(248), inset: 10pt, radius: 4pt, width: 100%,
)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #text(fill: rgb("#888888"))[q> ]#raw(input)\
  #raw(output)
]

// Usage:
#repl("partition_gf(10)",
  "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)")
```

### in-dexter Index Usage
```typst
#import "@preview/in-dexter:0.7.2": *

// Mark an index entry in running text
The #index[q-Pochhammer symbol]q-Pochhammer symbol $(a;q)_n$ is...

// Nested sub-entry
This uses the #index("products", "Jacobi triple")Jacobi triple product...

// Main entry (bold page number)
#index-main[aqprod]

// Cross-reference Maple names in migration chapter
#index("Maple functions", "numbpart")
#index("Maple functions", "qphihyper")

// Generate the index at the end
#make-index(title: [Index])
```

### GitHub Actions: Compile Typst and Attach to Release
```yaml
  build-manual:
    name: Build PDF Manual
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: typst-community/setup-typst@v4
      - name: Compile PDF
        run: typst compile manual/main.typ manual/q-kangaroo-manual.pdf
      - uses: actions/upload-artifact@v4
        with:
          name: manual-pdf
          path: manual/q-kangaroo-manual.pdf
```

## Discretion Recommendations

### Function Reference Grouping: Group by Domain (recommended)
**Rationale:** The 81 functions naturally cluster into 8 domain groups (Products, Partitions, Theta, Series Analysis, Relations, Hypergeometric, Mock Theta/Bailey, Identity Proving) as already organized in `help.rs`. Domain grouping provides mathematical context -- a researcher reading about `jacprod` benefits from seeing `tripleprod` and `quinprod` nearby. Within each group, alphabetical order. A flat A-Z listing would scatter related functions (e.g., `bailey_apply_lemma` at the start, `bailey_chain` nearby, but `bailey_discover` and `bailey_weak_lemma` pages later).

### Page Layout: Single Column
**Rationale:** Mathematical content with display equations and REPL transcript blocks needs full page width. Two-column layouts cause awkward equation wrapping and code block truncation.

### File Organization: Multi-File Split by Chapter
**Rationale:** At 100+ pages with 15 logical chapters, multi-file is essential. Each chapter in its own `.typ` file enables independent editing. The `main.typ` file handles global settings and `#include` ordering.

### Font: New Computer Modern (body) + DejaVu Sans Mono (code)
**Rationale:** New Computer Modern is the standard academic/mathematical font, familiar to the target audience. DejaVu Sans Mono is Typst's bundled monospace font -- no external font installation needed.

### Worked Examples: 4-6 Examples
**Rationale:** Cover each major capability area:
1. **Euler's pentagonal theorem** -- Products + identity verification (introductory)
2. **Ramanujan congruences p(5n+4)** -- Partitions + sift + findcong (research workflow)
3. **Jacobi triple product identity** -- Theta + products + prove_eta_id (identity proving)
4. **Rogers-Ramanujan identities** -- Bailey chains + search_identities (advanced)
5. **Hypergeometric transformation chain** -- phi + heine + find_transformation_chain (exploration)
6. **Mock theta function relations** -- mock_theta + findlincombo (modern research)

## Content Extraction Sources

The following authoritative source files provide all content for the manual:

| Content | Source File | What to Extract |
|---------|-------------|-----------------|
| All 81 function signatures | `crates/qsym-cli/src/eval.rs` lines 2295-2397 | `fn_signature()` match arms |
| All 81 function descriptions + examples | `crates/qsym-cli/src/help.rs` lines 128-732 | `FUNC_HELP` array |
| Function grouping (8 groups) | `crates/qsym-cli/src/help.rs` lines 13-107 | `general_help()` text |
| Maple alias mappings (15 aliases) | `crates/qsym-cli/src/eval.rs` lines 2383-2397 | Alias -> canonical name map |
| CLI flags and modes | `crates/qsym-cli/src/main.rs` lines 119-143 | `print_usage()` text |
| Exit codes (7 codes) | `crates/qsym-cli/src/script.rs` lines 15-27 | `EXIT_*` constants |
| Session commands (7 commands) | `crates/qsym-cli/src/commands.rs` lines 19-35 | `Command` enum |
| Expression language (operators, types) | `crates/qsym-cli/src/token.rs` + `ast.rs` | Token/AST enums |
| Value types (10 types) | `crates/qsym-cli/src/eval.rs` lines 28-49 | `Value` enum |
| Mathematical notation reference | `docs/mathematical_notation.rst` | Existing LaTeX definitions |
| Maple migration mappings | `docs/examples/maple_migration.ipynb` cell 46 | Complete mapping table |
| Existing vignettes (content source) | `docs/examples/*.ipynb` | 8 topic notebooks |

## Expression Language Reference Content

The expression language chapter needs to document:

| Feature | Syntax | Notes |
|---------|--------|-------|
| Integer literals | `42`, `999999999999` | Arbitrary precision (BigInteger for > i64) |
| The q indeterminate | `q` | Reserved keyword, not a variable |
| Infinity keyword | `infinity` | Used as argument to aqprod |
| String literals | `"filename.qk"` | For read() and save commands |
| Variable assignment | `name := expr` | `:=` operator |
| Last result | `%` | References previous result |
| Addition | `a + b` | Series, integer, rational |
| Subtraction | `a - b` | |
| Multiplication | `a * b` | |
| Division | `a / b` | |
| Exponentiation | `a ^ n` | Integer exponent only for series |
| Unary negation | `-a` | |
| Function calls | `f(a, b, c)` | 81 built-in functions |
| List literals | `[a, b, c]` | Used for findlincombo candidates, etc. |
| Statement terminator (print) | `expr;` or implicit | Print result |
| Statement terminator (suppress) | `expr:` | Suppress output |
| Comments | `# comment` | Line comments |
| Multi-statement | `a := 1; b := 2; a + b` | Semicolons separate |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| LaTeX for math PDFs | Typst gaining adoption | 2023-present | Faster compilation, simpler syntax, single binary |
| Manual index in LaTeX | in-dexter package for Typst | 2024 | Declarative index entries, automatic page tracking |
| Custom CI scripts | typst-community/setup-typst@v4 | 2024 | Standard GitHub Action with caching |

## DOC-06: --help Mentions PDF Manual

The `print_usage()` function in `crates/qsym-cli/src/main.rs` (lines 120-143) needs one additional line mentioning the PDF manual. This is the only code change in this phase. Example:

```
DOCUMENTATION:
  See the q-Kangaroo Reference Manual (PDF) included with release downloads.
```

## Open Questions

1. **Typst semicolon behavior in $(a;q)_n$**
   - What we know: Semicolons in Typst math mode create array arguments inside function calls. In bare math expressions, they appear to render correctly.
   - What's unclear: Whether complex nested expressions with semicolons (e.g., `$(a;q)_n (b;q)_n / (c;q)_n$`) cause any layout issues.
   - Recommendation: Test early in phase execution. If problematic, define `#let qpoch(a, n) = $(#a; q)_#n$` as a custom command.

2. **Exact page count**
   - What we know: User wants 100+ pages. With 81 functions at ~1 page each, plus 4 intro chapters, worked examples, migration table, and index, 120-140 pages is realistic.
   - What's unclear: Whether the full treatment of each function averages closer to 1 or 1.5 pages.
   - Recommendation: Don't pad; let content drive length. The 81 detailed function entries alone should hit 100+ pages.

3. **in-dexter cross-reference ("see also") support**
   - What we know: in-dexter supports sub-entries and main entries. The README does not mention "see also" cross-references.
   - What's unclear: Whether "see also" can be approximated with manual entries.
   - Recommendation: For Maple name cross-references, use sub-entries under a "Maple functions" top-level entry (e.g., "Maple functions > numbpart, see partition_count"). This achieves the user's goal without requiring cross-reference support.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-cli/src/help.rs` -- All 81 function help entries, grouping, signatures
- `crates/qsym-cli/src/eval.rs` -- Function dispatch, signatures, Maple aliases, Value enum
- `crates/qsym-cli/src/main.rs` -- CLI modes, flags, print_usage()
- `crates/qsym-cli/src/commands.rs` -- Session commands
- `crates/qsym-cli/src/token.rs` + `ast.rs` -- Expression language grammar
- `crates/qsym-cli/src/script.rs` -- Exit codes
- `docs/mathematical_notation.rst` -- Mathematical definitions
- `docs/examples/maple_migration.ipynb` -- Complete Maple mapping table
- [Typst releases (GitHub)](https://github.com/typst/typst/releases) -- v0.14.2 confirmed current
- [Typst math documentation](https://typst.app/docs/reference/math/) -- Math syntax reference
- [Typst equation documentation](https://typst.app/docs/reference/math/equation/) -- Numbering, display modes
- [Typst outline documentation](https://typst.app/docs/reference/model/outline/) -- TOC generation

### Secondary (MEDIUM confidence)
- [in-dexter GitHub](https://github.com/RolfBremer/in-dexter) -- Index package v0.7.2, verified via GitHub
- [typst-community/setup-typst](https://github.com/typst-community/setup-typst) -- GitHub Action v4
- [Typst multi-file discussion](https://github.com/typst/typst/discussions/2201) -- #include patterns

### Tertiary (LOW confidence)
- Typst semicolon behavior in Pochhammer notation -- needs validation during execution

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Typst v0.14.2 confirmed, in-dexter v0.7.2 confirmed, setup-typst@v4 confirmed
- Architecture: HIGH -- Multi-file Typst pattern well-documented, all 81 functions catalogued from source
- Content sources: HIGH -- All authoritative source files identified and read
- Typst math syntax: MEDIUM -- Basic syntax confirmed from docs, complex q-series notation needs in-phase validation
- Pitfalls: HIGH -- Common Typst pitfalls researched, CLI vs Python API confusion well-understood

**Research date:** 2026-02-18
**Valid until:** 2026-03-18 (Typst is fast-moving but core features are stable)
