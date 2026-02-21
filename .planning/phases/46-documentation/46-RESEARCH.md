# Phase 46: Documentation - Research

**Researched:** 2026-02-21
**Domain:** Typst PDF manual authoring, REPL help system, worked examples for v3.0 features
**Confidence:** HIGH

## Summary

Phase 46 is a documentation-only phase that adds no new functionality. The work covers three areas: (1) a new manual chapter documenting the scripting language added in v3.0 (for-loops, procedures, if/elif/else/fi, boolean/comparison operators, plus series/expand/factor/subs and bivariate product functions), (2) three new help entries in the REPL (`help for`, `help proc`, `help if`) plus tab-completion keywords, and (3) three worked examples reproducing qmaple.pdf tutorial patterns woven into the scripting chapter.

The existing codebase provides all the patterns needed. The manual has 15 Typst chapter files using a well-defined template system (`repl`, `repl-block`, `func-entry`, `index`, `index-main`). The help system in `help.rs` uses a `FuncHelp` struct array for per-function help and a `general_help()` function for the overview. Tab completion in `repl.rs` has a `canonical_function_names()` list and `check_keyword()` for multi-line detection. All patterns are established and consistent.

One important finding: Chapter 4 (Expression Language) currently states "There are no control-flow statements (loops, conditionals)" which is now false. This must be updated. The function count "89" in multiple locations is also outdated (now 95 functions in the help system). Additionally, the `general_help()` text does not yet list `for`, `proc`, `if` as language features.

**Primary recommendation:** Create one new chapter file (e.g., `04b-scripting.typ` or renumber as appropriate) for the scripting language documentation, update Chapter 4's outdated text, add 3 help entries to `help.rs`, add missing tab-completion keywords to `repl.rs`, and weave 3 worked examples into the scripting chapter near the features they demonstrate.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Organize by workflow progression: simple -> complex (variables -> loops -> conditionals -> procedures -> series ops -> products)
- Tutorial + reference hybrid: brief explanatory text for each concept, then syntax and examples
- Bivariate series (tripleprod/quinprod/winquist with symbolic z) documented inline with product functions, not as a separate section
- Exactly 3 worked examples from qmaple.pdf
- One demonstrating for-loops with series computation
- One demonstrating procedure definitions with memoization
- One demonstrating tripleprod bivariate product identity
- q-Kangaroo code only (no Maple original), but cite the specific qmaple.pdf section each example reproduces
- Woven into the chapter near the features they demonstrate, not in a separate section at the end
- Only add missing help entries: `help for`, `help proc`, `help if` -- leave existing series/factor/subs help entries alone
- Short syntax summary + 1 example per entry (match existing help entry style)
- Add "See also:" cross-references linking related topics at the end of each entry
- Check and add missing keywords to tab completion (for, proc, if, elif, else, fi, od, end, local, RETURN) -- only add what's not already completable
- Match existing manual format for code blocks (Typst raw blocks with input + output)
- Minimal commentary -- let code speak for itself with syntax description above and example below
- Include a brief "Notes" subsection covering key differences from Maple where genuinely needed
- Cite specific qmaple.pdf section numbers for worked examples so readers can compare

### Claude's Discretion
- Whether to use one chapter or two (based on content length and logical grouping)
- Exact ordering of subsections within the workflow progression
- Which specific qmaple.pdf sections to cite for the 3 worked examples
- Whether any feature needs a "Notes" tip and what to include
- Loading skeleton / formatting details in Typst

### Deferred Ideas (OUT OF SCOPE)
None specified.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOC-01 | PDF manual includes a chapter on the scripting language (for, proc, if, series, factor, subs) | New Typst chapter file using established `repl`, `repl-block`, `func-entry`, `index` template helpers. Follow workflow progression. Update Chapter 4 outdated text. |
| DOC-02 | Help system (`help for`, `help proc`, `help if`) documents new scripting syntax | Add entries to `help.rs` -- either as new items in `FUNC_HELP` array or as special cases in `function_help()`. Add `general_help()` text. Add tab-completion keywords in `repl.rs`. |
| DOC-03 | Worked examples section includes reproductions of key examples from Garvan's qmaple.pdf tutorial | 3 examples woven into the scripting chapter: (1) for-loop series computation, (2) memoized procedure, (3) tripleprod bivariate identity. Cite qmaple.pdf section numbers. |
</phase_requirements>

## Standard Stack

### Core (No New Dependencies)
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Typst | 0.14.2 | PDF manual compilation | Already used; all chapters authored in Typst |
| in-dexter | 0.7.2 | Back-of-book index generation | Already imported in template.typ |
| Rust (help.rs) | 1.85 | REPL help system | Existing `FuncHelp` struct pattern |
| Rust (repl.rs) | 1.85 | Tab completion and multi-line validation | Existing `ReplHelper` pattern |

### No New Crates Required
This phase modifies only `.typ` files and two existing `.rs` files. No new Rust or Typst dependencies.

## Architecture Patterns

### Recommended File Changes

```
manual/
  main.typ                    -- Add #include for new chapter(s), update function count
  chapters/
    04-expression-language.typ -- Update: remove "no control-flow" text, update function count
    04b-scripting.typ          -- NEW: Scripting Language chapter (or use different numbering)
    05-products.typ            -- Add bivariate section for tripleprod/quinprod/winquist symbolic z
    00-title.typ               -- Update function count from 89 to 97

crates/qsym-cli/src/
  help.rs      -- Add `help for`, `help proc`, `help if` entries; update general_help()
  repl.rs      -- Add missing keywords to tab completion; tests
```

### Pattern 1: New Typst Chapter File

**What:** Create a new chapter documenting the scripting language (for/proc/if/series/factor/subs).
**When to use:** For DOC-01.
**Rationale for one chapter:** The scripting language features (loops, procedures, conditionals) plus the new expression/polynomial operations (series, expand, factor, subs) form a coherent workflow progression from simple to complex. One chapter of approximately 8-12 pages keeps them together. If content exceeds 15 pages, consider splitting into "Scripting Language" and "Expression & Polynomial Operations".

**Structure template:**
```typst
// 04b-scripting.typ -- Scripting Language
#import "../template.typ": *

= Scripting Language
#index[scripting language]

[Brief intro paragraph]

== For Loops
#index[for loops]
[Syntax, examples, worked example #1 woven in]

== If/Elif/Else Conditionals
#index[conditionals]
[Syntax, examples]

== Boolean and Comparison Operators
#index[boolean operators]
#index[comparison operators]
[Table of operators, examples]

== Procedures
#index[procedures]
[proc/end, local, option remember, RETURN, worked example #2 woven in]

== Expression Operations
#index[expression operations]
[series(), expand() -- func-entry style]

== Polynomial Operations
#index[polynomial operations]
[factor(), subs() -- func-entry style]
```

### Pattern 2: Bivariate Documentation Inline with Products

**What:** Add documentation of symbolic z behavior to the existing product function entries in Chapter 5 (Products).
**When to use:** For the tripleprod/quinprod/winquist bivariate behavior.
**Approach:** The help entries for tripleprod/quinprod/winquist already mention bivariate behavior (updated in Phase 45). The manual chapter 05-products.typ should get additional examples showing symbolic z usage. The tripleprod bivariate worked example (example #3) goes in the scripting chapter near the product operations or as a standalone subsection.

**Per CONTEXT.md:** Bivariate series documented inline with product functions, not as a separate section. The worked example demonstrating tripleprod bivariate identity is woven into the chapter near the features it demonstrates.

### Pattern 3: Help Entry Addition

**What:** Add special help entries for `for`, `proc`, `if` that are language constructs, not functions.
**Implementation:** These are not function calls, so they should be handled as special cases in `function_help()` rather than as `FuncHelp` entries (which have a signature/example pattern designed for functions). Add match arms directly:

```rust
pub fn function_help(name: &str) -> Option<String> {
    // Language construct help (not function calls)
    match name {
        "for" => return Some(String::from(
            "for var from start to end [by step] do body od\n\n  ..."
        )),
        "proc" => return Some(String::from(
            "name := proc(params) [local vars;] [option remember;] body; end\n\n  ..."
        )),
        "if" => return Some(String::from(
            "if cond then body [elif cond then body] [else body] fi\n\n  ..."
        )),
        _ => {}
    }
    // ... existing function lookup
}
```

Also update `general_help()` to list these under a "Scripting:" category.

### Pattern 4: Tab Completion Keywords

**What:** Add missing keywords that users might want to tab-complete.
**Current state of `canonical_function_names()`:** Contains only function names (91 entries). Keywords like `for`, `proc`, `if`, etc. are NOT in this list. They are also NOT in `command_names`.
**Current state of `command_names`:** `["help", "quit", "exit", "clear", "restart", "set", "latex", "save"]`
**What's already completable via the keyword/validator system:** Nothing -- the validator tracks `for`/`od`/`if`/`fi`/`proc`/`end` for multi-line detection but does NOT add them to completion candidates.

**Keywords to add to completion:** `for`, `proc`, `if`, `elif`, `else`, `fi`, `od`, `end`, `local`, `RETURN`

**Implementation:** Add these as a new `keyword_names` list in `ReplHelper` alongside `function_names` and `command_names`. Complete keywords without `(` suffix (unlike functions). Or simply add `RETURN` to `function_names` (since it uses function-call syntax) and the rest to a new keywords list.

### Anti-Patterns to Avoid
- **Creating a separate "Bivariate Series" chapter:** CONTEXT.md explicitly says bivariate documentation goes inline with product functions.
- **Putting worked examples in a separate section at the end:** CONTEXT.md says woven into the chapter near the features they demonstrate.
- **Modifying existing help entries for series/factor/subs:** CONTEXT.md says leave existing help entries alone; only add missing ones.
- **Including Maple original code alongside q-Kangaroo:** CONTEXT.md says q-Kangaroo code only.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| REPL transcript formatting | Manual monospace text | `#repl()` and `#repl-block()` from template.typ | Consistent styling with existing chapters |
| Function reference entries | Ad-hoc formatting | `#func-entry()` from template.typ | Standardized layout with params/examples/related |
| Index entries | None | `#index[]` and `#index-main[]` from in-dexter | Back-of-book index generation |
| Help entry formatting | Custom formatting | Existing `FuncHelp` struct or `function_help()` match arms | Consistent with 95 existing entries |

## Common Pitfalls

### Pitfall 1: Outdated Function Counts
**What goes wrong:** Manual says "89 functions" in 5+ locations but the actual count is 95 (after v2.0 additions).
**Why it happens:** v2.0 added series, expand, factor, subs, floor, legendre but the manual text was not updated.
**How to avoid:** Search for "89" in all `.typ` files and update to the correct count. Current count: 95 canonical functions + RETURN as a pseudo-function = 95 in help, 97 counting `anames` and `restart` session functions that were added later. Use whatever count matches `general_help()`.
**Locations to update:** `00-title.typ` (line 21), `01-quick-start.typ` (line 84), `02-installation.typ` (line 69), `03-cli-usage.typ` (line 129), `04-expression-language.typ` (line 130).

### Pitfall 2: Chapter Numbering Collision
**What goes wrong:** Adding a new chapter between 04 and 05 requires either renumbering files or using a non-numeric name.
**Why it happens:** Files are numbered `04-expression-language.typ`, `05-products.typ`.
**How to avoid:** Use a name like `04b-scripting.typ` to insert between 04 and 05 without renumbering. Alternatively, renumber everything from 05 onward. The simplest approach is `04b-scripting.typ` with `#include "chapters/04b-scripting.typ"` in main.typ after the chapter 04 include.
**Warning signs:** If renumbering, every cross-reference like "See Chapter 5" needs updating.

### Pitfall 3: Expression Language Chapter Outdated Intro
**What goes wrong:** Chapter 4 intro says "There are no control-flow statements (loops, conditionals)".
**Why it happens:** Written before v3.0 added for/proc/if.
**How to avoid:** Update the paragraph to mention that control flow is now supported, with a forward reference to the new scripting chapter. Also update the Value Types table to include Symbol and Procedure types.

### Pitfall 4: Help Entry Test Count Assertions
**What goes wrong:** Tests assert exact counts: `assert_eq!(canonical.len(), 95)` and `assert_eq!(FUNC_HELP.len(), 95)`. Adding help entries without updating these counts causes test failures.
**Why it happens:** Strict count assertions in `help.rs` tests.
**How to avoid:** If adding `for`/`proc`/`if` as special cases in `function_help()` (not as `FUNC_HELP` array entries), the count assertions remain correct. If adding to `FUNC_HELP`, update both the count and the `canonical` test vector. The special-case approach avoids this entirely.

### Pitfall 5: REPL Tab Completion Test Assumptions
**What goes wrong:** Adding keywords to completion could break tests that assume certain prefix matches.
**Why it happens:** Tests check specific completion results for prefixes like "q", "aq", "theta".
**How to avoid:** Review existing completion tests before adding new keywords. Keywords like `for`, `proc`, `if` are short and could match prefixes in unexpected ways. Add specific tests for the new keywords.

### Pitfall 6: Worked Example Output Accuracy
**What goes wrong:** Example output in the manual doesn't match what the REPL actually produces.
**Why it happens:** Typos in output strings, or format changes since v2.0.
**How to avoid:** Run each example in the actual REPL before writing it into the manual. Capture exact output. Series display uses descending degree order with `O(q^T)` suffix.

## Code Examples

### Example 1: Existing Typst repl-block Usage (from 01-quick-start.typ)
```typst
#repl-block("q> f := aqprod(q, q, infinity, 20):
q> f
1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)")
```

### Example 2: Existing func-entry Usage (from 05-products.typ)
```typst
#func-entry(
  name: "series",
  signature: "series(expr, q, T)",
  description: [
    Truncate a $q$-series to $O(q^T)$. ...
    #index[series truncation]
  ],
  params: (
    ([expr], [Series/JacobiProduct/Integer], [The expression to truncate]),
    ([q], [Variable], [The series variable]),
    ([T], [Integer], [New truncation order]),
  ),
  examples: (
    ("f := aqprod(q, q, infinity, 50): series(f, q, 10)",
     "-q^7 - q^5 + q^2 + q + 1 + O(q^10)"),
  ),
  related: ("expand",),
)
```

### Example 3: Help Entry Special Case Pattern
```rust
pub fn function_help(name: &str) -> Option<String> {
    // Language construct help entries
    match name {
        "for" => return Some(String::from(
            "for var from start to end [by step] do body od\n\n\
             \x20 Execute body repeatedly with var taking values start, start+step, ..., end.\n\
             \x20 Default step is 1. Body statements are separated by ; or :\n\n\
             \x20 Example:\n\
             \x20   q> s := 0: for k from 1 to 5 do s := s + k od; s\n\
             \x20   15\n\n\
             \x20 See also: if, proc"
        )),
        // ... proc, if entries
        _ => {}
    }
    // Redirect aliases to canonical names
    let lookup = match name {
        "partition_count" => "numbpart",
        "L" => "legendre",
        _ => name,
    };
    FUNC_HELP.iter().find(|h| h.name == lookup).map(|h| { ... })
}
```

### Example 4: Tab Completion Keyword Addition
```rust
impl ReplHelper {
    pub fn new() -> Self {
        Self {
            function_names: Self::canonical_function_names(),
            keyword_names: vec![
                "for", "from", "to", "by", "do", "od",
                "if", "then", "elif", "else", "fi",
                "proc", "local", "end",
                "RETURN",
                "and", "or", "not",
            ],
            command_names: vec!["help", "quit", "exit", "clear", "restart", "set", "latex", "save"],
            var_names: Vec::new(),
        }
    }
    // In complete_inner(), add keyword completion alongside function names
    // Keywords complete without trailing '(' (unlike functions)
}
```

## Worked Example Selection (qmaple.pdf Citations)

The CONTEXT.md requires exactly 3 worked examples from Garvan's qmaple.pdf. Based on the research from Phase 42 and the known content of qmaple.pdf (arXiv:math/9812092, "A q-Product Tutorial for a q-Series Maple Package"):

### Example 1: For-Loop with Series Computation
**Demonstrates:** for-loop, variable accumulation, series arithmetic
**qmaple.pdf reference:** Section 3 (Euler function computations) or Section 4 (product factorizations) -- any section that uses a loop to build up series. The UE (unit Eisenstein series) procedure from the qmaple tutorial uses nested for-loops with `L(m,p)` (Legendre symbol) and series accumulation.
**q-Kangaroo adaptation:**
```
# Compute partial sums of generalized pentagonal series
s := 0:
for n from -5 to 5 do
  s := s + (-1)^n * q^(n*(3*n-1)/2):
od:
series(s, q, 20)
```
This demonstrates: for-loop with negative range, runtime exponent arithmetic (`n*(3*n-1)/2`), series accumulation, `series()` truncation.

### Example 2: Procedure with Memoization
**Demonstrates:** proc/end, local variables, option remember, RETURN, if/else, recursion
**qmaple.pdf reference:** The tutorial discusses procedures with `option remember` for memoized computations. The classic Fibonacci example parallels Garvan's approach to caching intermediate q-series computations.
**q-Kangaroo adaptation:**
```
# Memoized partition counting via Euler's recurrence
prec := proc(n)
  option remember;
  local k, s;
  if n < 0 then RETURN(0) fi;
  if n = 0 then RETURN(1) fi;
  s := 0:
  for k from 1 to n do
    if k*(3*k-1)/2 > n then RETURN(s) fi;
    s := s + (-1)^(k+1) * (prec(n - k*(3*k-1)/2) + prec(n - k*(3*k+1)/2)):
  od:
  s;
end:
prec(20)
```
This demonstrates: procedure definition, local scoping, memoization, early RETURN, nested for-loop + if, recursive calls.

### Example 3: Tripleprod Bivariate Product Identity
**Demonstrates:** tripleprod with symbolic z, bivariate series display, identity verification
**qmaple.pdf reference:** Section on triple product identities (Jacobi triple product). The tutorial shows `tripleprod(z,q,T)` and verification of the Jacobi triple product identity sum_{n=-inf}^{inf} (-1)^n z^n q^{n(n-1)/2}.
**q-Kangaroo adaptation:**
```
# Jacobi triple product: verify sum vs product form
tp := tripleprod(z, q, 10)
```
This demonstrates: symbolic z argument producing bivariate Laurent polynomial, verifying the classical identity by examining z-coefficient structure.

**Note on citations:** The exact section numbers in qmaple.pdf should be verified by the plan executor since the PDF is not directly parseable. The paper is structured around product computations (Sections 2-4), product analysis (Sections 5-7), and identity discovery (Sections 8-10). The recommended citations are:
- Example 1: "cf. Garvan, qmaple.pdf, Section 4 (Euler's pentagonal theorem computations)"
- Example 2: "cf. Garvan, qmaple.pdf, Section 7 (q-series procedures with memoization)"
- Example 3: "cf. Garvan, qmaple.pdf, Section 2 (Jacobi triple product)"

These should be confirmed/adjusted during execution by checking the actual PDF content.

## Detailed Inventory of Changes

### Files to Create
| File | Est. Lines | Content |
|------|-----------|---------|
| `manual/chapters/04b-scripting.typ` | 200-300 | Scripting language chapter with 6-8 subsections + 3 woven worked examples |

### Files to Modify
| File | Change | Est. Lines Changed |
|------|--------|-------------------|
| `manual/main.typ` | Add `#include "chapters/04b-scripting.typ"` after line 50 | 1 |
| `manual/chapters/04-expression-language.typ` | Update "no control-flow" text (line 15-17), update function count (line 130), add forward reference to scripting chapter, update Value Types table to include Symbol and Procedure | 15-20 |
| `manual/chapters/00-title.typ` | Update "89" to correct count | 1 |
| `manual/chapters/01-quick-start.typ` | Update "89" to correct count | 1 |
| `manual/chapters/02-installation.typ` | Update "89" to correct count | 1 |
| `manual/chapters/03-cli-usage.typ` | Update "89" to correct count | 1 |
| `crates/qsym-cli/src/help.rs` | Add `for`/`proc`/`if` help entries in `function_help()`, update `general_help()` text | 40-50 |
| `crates/qsym-cli/src/repl.rs` | Add keyword completion support, tests | 30-40 |

### Files NOT Modified
- `manual/chapters/05-products.typ` -- Bivariate behavior already mentioned in help entries (updated Phase 45). Per CONTEXT.md, bivariate docs go inline with product functions. The tripleprod/quinprod/winquist `func-entry` blocks already describe symbolic z behavior. The worked example for bivariate goes in the scripting chapter.
- `manual/chapters/13-worked-examples.typ` -- Per CONTEXT.md, worked examples are woven into the scripting chapter, not added to the existing worked examples chapter.
- Existing help entries for series/factor/subs -- Per CONTEXT.md, leave alone.

## State of the Art

| Old State | Current State | Impact |
|-----------|--------------|--------|
| Chapter 4 says "no control-flow" | v3.0 added for/proc/if | Must update Chapter 4 text |
| Manual says "89 functions" | Actually 95 functions in help system | Must update 5 locations |
| No help for language constructs | Need `help for`, `help proc`, `help if` | Add 3 entries to help.rs |
| Tab completion: functions + commands only | Keywords not completable | Add ~15 keywords to repl.rs |
| Worked examples chapter: 6 examples (math-focused) | Need 3 scripting-focused examples from qmaple.pdf | Add to new scripting chapter |

## Open Questions

1. **Exact function count to display**
   - What we know: help.rs has 95 `FuncHelp` entries. `canonical_function_names()` in repl.rs has 91 entries. `general_help()` lists functions in 12 categories. The discrepancy is because `FUNC_HELP` has 95 entries while the repl completion has 91 function names (some help entries cover functions not in completion, e.g., `anames`/`restart` are session commands not in function_names).
   - What's unclear: Whether to count `RETURN` as a function, whether to count session commands (`anames`, `restart`), etc.
   - Recommendation: Use "97" for the total number of built-in functions counting RETURN, anames, restart, read as functions (matching what general_help lists). Or keep the help-system authoritative count. The executor should count what `general_help()` actually lists and use that number.

2. **Exact qmaple.pdf section numbers**
   - What we know: The paper is arXiv:math/9812092 with ~25 pages covering products, series analysis, and identities.
   - What's unclear: Exact section numbers for the 3 examples (PDF not directly parseable).
   - Recommendation: During execution, use approximate section references like "cf. Garvan [qmaple.pdf], Section X" with the executor verifying or using best-guess section numbers. If exact verification is impossible, cite the paper generically: "cf. Garvan, 'A q-Product Tutorial for a q-Series Maple Package' (1998)".

3. **Whether Chapter 4 Value Types table needs updating**
   - What we know: The table lists 10 types (Series through Infinity). v3.0 added Symbol, JacobiProduct, Procedure, BivariateSeries, TrivariateSeries as Value variants.
   - What's unclear: Whether all new types should be documented or only user-facing ones.
   - Recommendation: Add Symbol and Procedure to the Value Types table. BivariateSeries and TrivariateSeries can be mentioned briefly. JacobiProduct is already implicitly covered via JAC documentation.

## Sources

### Primary (HIGH confidence)
- **Codebase inspection:** All files in `manual/chapters/*.typ`, `manual/main.typ`, `manual/template.typ` -- read directly and patterns documented
- **Codebase inspection:** `crates/qsym-cli/src/help.rs` (1195 lines) -- complete help system with 95 FuncHelp entries
- **Codebase inspection:** `crates/qsym-cli/src/repl.rs` (470+ lines) -- tab completion and multi-line validator
- **Codebase inspection:** `crates/qsym-cli/src/ast.rs` -- ForLoop, IfExpr, ProcDef AST nodes
- **Codebase inspection:** `crates/qsym-cli/src/eval.rs` -- Procedure struct, eval_for_loop, eval_if_expr, call_procedure implementations
- **Codebase inspection:** `crates/qsym-cli/src/token.rs` -- all keyword tokens (For, From, To, By, Do, Od, If, Then, Elif, Else, Fi, Proc, Local, OptionKw, End)
- **46-CONTEXT.md** -- All user decisions locked

### Secondary (MEDIUM confidence)
- Phase 42 RESEARCH.md -- Procedure architecture patterns, Garvan UE procedure example
- Phase 43 RESEARCH.md -- series/expand/floor/legendre implementation details
- Phase 45 RESEARCH.md -- BivariateSeries architecture, symbolic z detection
- [Garvan qmaple.pdf](https://qseries.org/fgarvan/papers/qmaple.pdf) -- Source for worked example citations (arXiv:math/9812092)

### Tertiary (LOW confidence)
- qmaple.pdf section numbers -- Could not directly parse the PDF; section number citations are approximate and should be verified during execution.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, pure documentation work
- Architecture: HIGH - All patterns established in existing chapters and help system, directly observed
- Pitfalls: HIGH - All identified from direct codebase analysis (outdated text, count mismatches, test assertions)
- Worked examples: MEDIUM - Example code is straightforward but qmaple.pdf section citations are approximate

**Research date:** 2026-02-21
**Valid until:** No expiration (documentation of existing stable features)
