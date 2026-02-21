# Phase 51: Documentation - Research

**Researched:** 2026-02-21
**Domain:** Help system, tab completion, and PDF manual for v4.0 features
**Confidence:** HIGH

## Summary

Phase 51 is a documentation-only phase covering all v4.0 features implemented in Phases 47-50. The codebase has well-established patterns for all three documentation surfaces: (1) help entries in `help.rs` using the `FuncHelp` struct and `function_help()` match arms, (2) tab completion in `repl.rs` via `canonical_function_names()` and `keyword_names`, and (3) PDF manual chapters in Typst using the `#func-entry`, `#repl`, `#repl-block`, and `#index` macros from `template.typ`.

The investigation reveals a precise gap analysis: help.rs currently has 97 function help entries and 3 language-construct help entries (for, proc, if). New help entries are needed for `ditto`, `lambda`/arrow syntax, `radsimp`, and potentially `read`. The tab completion list in repl.rs is missing `min`, `max`, and `radsimp` (plus `read`). The PDF manual needs a new chapter covering all 14 v4.0 changes organized by type. The qmaple.pdf (Garvan 1998, 25 pages) contains executable examples from Sections 3-6 that can be reproduced in q-Kangaroo, forming the basis for the walkthrough section.

**Primary recommendation:** Follow the exact same patterns used in previous documentation phases -- `FuncHelp` structs for functions, match arms for language constructs, Typst `#func-entry` for the manual -- adding entries for each v4.0 feature with examples drawn from qmaple.pdf where applicable.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Full help entries with signature, description, 2-3 examples, and edge case notes
- Examples taken directly from qmaple.pdf so users can cross-reference
- No Maple references in help text -- entries should be self-contained
- Update existing help entries for functions that got new signatures in v4.0 (theta3 2-arg, qfactor 2-arg, aqprod 3-arg)
- Include help for both min() and max()
- Organize v4.0 chapter by feature type: Language Features, Bug Fixes, New Functions
- Include a full walkthrough section reproducing ALL executable qmaple.pdf examples
- Include a "Not Yet Supported" subsection listing examples that require deferred features (while loops, zqfactor, etc.)
- Examples show commands + expected output (REPL transcript style)
- Match qmaple.pdf presentation style -- same sequence of commands in same order
- Reference qmaple.pdf by both section number and page: "Section 3.2 (p.12)"
- Add function names only to tab completion: jac2series, radsimp, quinprod, subs, min, max
- No keyword completions (prodid/seriesid are just string arguments)

### Claude's Discretion
- Which features get help entries (all new functions certainly, language syntax features at Claude's judgment)
- Whether arrow/ditto get special completion behavior
- How to organize multi-feature examples (grouped in walkthrough vs duplicated per feature)

### Deferred Ideas (OUT OF SCOPE)
- None specified
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Typst | 0.14.2 | PDF manual compilation | Already used for entire manual |
| in-dexter | 0.7.2 | Back-of-book index generation | Already imported in template.typ |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| template.typ | internal | `#func-entry`, `#repl`, `#repl-block`, `#index` macros | All manual content |

No new libraries needed. This phase edits only three existing source files and adds one new Typst chapter.

## Architecture Patterns

### Pattern 1: Function Help Entries (help.rs)

**What:** Each function has a `FuncHelp` struct in the `FUNC_HELP` array. Language constructs (for, proc, if) use special-case match arms in `function_help()`.

**When to use:** For every new function that needs `?name` help.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/help.rs`

```rust
// For functions: add to FUNC_HELP array
FuncHelp {
    name: "radsimp",
    signature: "radsimp(expr)",
    description: "Simplify a rational series expression...",
    example: "q> radsimp(theta3(q,100)^2/theta3(q^5,100)^2)",
    example_output: "...",
},

// For language constructs: add match arm in function_help()
"ditto" | "\"" => return Some(String::from(
    "\" (ditto operator)\n\n\
     \x20 Reference the last computed result...\n\n\
     \x20 Example:\n\
     \x20   q> aqprod(q,q,10); etamake(\",q,100)\n\
     \x20   ...\n\n\
     \x20 See also: %"
)),
```

**Key details:**
- The `FuncHelp` struct has 5 fields: `name`, `signature`, `description`, `example`, `example_output`
- Language construct help uses raw `String::from()` with `\x20` for leading spaces
- The `general_help()` function must also be updated to list new functions in the correct category
- Existing test `every_canonical_function_has_help_entry` validates 97 canonical names
- Test `func_help_count_matches_canonical` asserts `FUNC_HELP.len() == 97`

### Pattern 2: Tab Completion (repl.rs)

**What:** `canonical_function_names()` returns all function names that tab-complete with auto-paren `(`. `keyword_names` lists keywords that complete without paren.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/repl.rs`

```rust
fn canonical_function_names() -> Vec<&'static str> {
    vec![
        // ... existing entries ...
        // Group P: Number Theory -- add min, max here
        "floor", "legendre", "min", "max",
        // Group T: Simplification
        "radsimp",
    ]
}
```

**Key details:**
- Test `canonical_function_count` asserts the list has exactly 97 entries (will need update)
- Test `no_duplicate_function_names` verifies no duplicates
- Functions complete with trailing `(`, keywords complete without
- The CONTEXT says NO keyword completions for prodid/seriesid -- they are just string arguments

### Pattern 3: PDF Manual Chapters (Typst)

**What:** Each chapter is a separate `.typ` file included from `main.typ`. Uses `#func-entry` for reference entries and `#repl`/`#repl-block` for REPL transcripts.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/manual/`

```typst
// New chapter file: chapters/16-v4-changes.typ
#import "../template.typ": *

= v4.0 Changes
#index[v4.0 changes]

== Language Features

=== Ditto Operator
#index[ditto operator]
// ... description and examples ...
#repl("aqprod(q,q,10); etamake(\",q,100)", "...")

== Bug Fixes
// ... etc ...

== New Functions
#func-entry(
  name: "radsimp",
  signature: "radsimp(expr)",
  description: [...],
  params: (([expr], [Series/Rational], [Expression to simplify]),),
  examples: (("radsimp(theta3(q,100)^2)", "..."),),
  related: ("subs", "series"),
)
```

**Key details:**
- `#func-entry` takes named params: `name`, `signature`, `description`, `params`, `examples`, `edge-cases`, `related`
- `#repl(input, output)` for single-line examples
- `#repl-block(content)` for multi-line REPL transcripts with `q> ` prompts
- `#index[term]` and `#index-main[term]` for index entries
- Chapter file must be `#include`d in `main.typ`

### Recommended Project Structure for Changes

```
crates/qsym-cli/src/
  help.rs        # Add entries + update general_help() + update tests
  repl.rs        # Add missing names to canonical_function_names() + update tests
manual/
  main.typ       # Add #include for new chapter
  template.typ   # Update version string to "1.0.0" (or appropriate)
  chapters/
    04-expression-language.typ   # May need updates for ditto, arrow, etc.
    16-v4-changes.typ            # NEW: v4.0 chapter
```

### Anti-Patterns to Avoid
- **Don't create separate help entries for aliases:** The `?qfactor` help already covers both 2-arg and 3-arg forms. Don't make a separate entry for `qfactor(f,T)`.
- **Don't add string arguments to tab completion:** `prodid` and `seriesid` are not function names -- they're string arguments to `quinprod()`. The CONTEXT explicitly says no keyword completions for these.
- **Don't mention Maple in help text:** The CONTEXT says entries should be self-contained. Reference qmaple.pdf in the manual only, not in help text.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| REPL transcript rendering | Custom Typst blocks | `#repl()` and `#repl-block()` macros from template.typ | Consistent styling, already tested |
| Function reference formatting | Manual heading/table layout | `#func-entry()` macro from template.typ | Handles params, examples, related links automatically |
| Index generation | Manual index | `#index[]`, `#index-main[]`, `#make-index()` from in-dexter 0.7.2 | Automatic back-of-book index |

## Common Pitfalls

### Pitfall 1: Test Count Assertions
**What goes wrong:** Adding entries to `FUNC_HELP` or `canonical_function_names()` without updating the corresponding count assertions causes test failures.
**Why it happens:** Multiple tests assert exact counts: `FUNC_HELP.len() == 97`, `canonical_function_names().len() == 97`, `every_canonical_function_has_help_entry` checks a hardcoded list of 97 names.
**How to avoid:** After adding entries, update ALL count assertions:
- `help.rs` test `func_help_count_matches_canonical` (currently asserts 97)
- `repl.rs` test `canonical_function_count` (currently asserts 97)
- `help.rs` test `every_canonical_function_has_help_entry` hardcoded list
- `eval.rs` test `function_count_verification` (asserts >= 78)
**Warning signs:** Tests fail with "expected 97, got 98" or "expected 97, got 100".

### Pitfall 2: Help for Language Constructs vs Functions
**What goes wrong:** Trying to add ditto/lambda/arrow as `FuncHelp` entries in the FUNC_HELP array. They aren't functions and the pattern doesn't fit.
**Why it happens:** These are syntax features, not callable functions.
**How to avoid:** Add them as match arms in `function_help()`, following the `"for"`, `"proc"`, `"if"` pattern. They bypass FUNC_HELP entirely.
**Warning signs:** The `name` field in FuncHelp doesn't match any dispatchable function name.

### Pitfall 3: general_help() Not Updated
**What goes wrong:** Users type `help` and don't see new functions listed. The per-function `?name` works but the overview is incomplete.
**Why it happens:** `general_help()` is a hardcoded string, separate from FUNC_HELP. It must be manually updated.
**How to avoid:** After adding any new function or category, update the `general_help()` string to include it. The existing test `general_help_contains_all_categories` and related tests help catch omissions.

### Pitfall 4: Tab Completion Mismatch with ALL_FUNCTION_NAMES
**What goes wrong:** Tab completion and the evaluator have different function lists, causing confusion where a function works but doesn't complete, or vice versa.
**Why it happens:** `canonical_function_names()` in repl.rs and `ALL_FUNCTION_NAMES` in eval.rs are maintained independently.
**How to avoid:** After modifying either list, verify both have the same entries. The comment in repl.rs says "must match eval.rs ALL_FUNCTION_NAMES exactly."

### Pitfall 5: qmaple.pdf Examples That Won't Work
**What goes wrong:** Including qmaple.pdf examples that require deferred features (while loops, `RootOf`, `with(numtheory)`, `with(qseries)`, `zqfactor`).
**Why it happens:** Not all Maple syntax is supported in q-Kangaroo.
**How to avoid:** Carefully classify each qmaple.pdf example as:
1. **Executable**: Can be directly typed at `q>` prompt (most Section 3-6 examples)
2. **Translatable**: Needs minor syntax changes (e.g., `>` prompt to `q>`)
3. **Not yet supported**: Requires deferred features (while loops, RootOf, etc.)
Include category 1 and 2 in the walkthrough. List category 3 in "Not Yet Supported" subsection.

## Code Examples

### Adding a Function Help Entry

```rust
// In help.rs FUNC_HELP array, add after existing entries:
FuncHelp {
    name: "radsimp",
    signature: "radsimp(expr)",
    description: "Simplify a rational expression involving series quotients.\n  Currently acts as the identity function, returning its input unchanged.\n  Provided for Maple compatibility where radsimp() is used to simplify\n  expressions with nested divisions.",
    example: "q> f := theta3(q, 20)^2: radsimp(f)",
    example_output: "4*q^9 + 4*q^4 + 4*q + 1 + ... + O(q^20)",
},
```

### Adding a Language Construct Help Entry

```rust
// In function_help() match block, before the _ => {} fallback:
"ditto" => return Some(String::from(
    "\" (ditto operator)\n\n\
     \x20 The double-quote character \" refers to the last printed result.\n\
     \x20 It is equivalent to % but matches Maple's convention.\n\n\
     \x20 Example:\n\
     \x20   q> aqprod(q, q, 5)\n\
     \x20   -q^15 + q^14 + ... + 1\n\
     \x20   q> etamake(\", q, 20)\n\
     \x20   eta(tau)\n\n\
     \x20 See also: %"
)),
```

### Adding Tab Completion Names

```rust
// In repl.rs canonical_function_names(), update Group P:
// Group P: Number Theory (4)
"floor", "legendre", "min", "max",
// Group T: Simplification (1)
"radsimp",
```

### Adding a Manual Chapter

```typst
// In main.typ, add before #include "chapters/15-appendix.typ":
#include "chapters/16-v4-changes.typ"

// In chapters/16-v4-changes.typ:
#import "../template.typ": *

= What's New in v4.0
#index[v4.0]

This chapter documents all changes introduced in q-Kangaroo v4.0...

== Language Features

=== Ditto Operator (\")
#index[ditto operator]

The double-quote character `"` references the last printed result...

#repl-block("q> aqprod(q, q, 5)
-q^15 + q^14 + q^13 - q^10 - q^9 - q^8 + q^7 + q^6 + q^5 - q^2 - q + 1
q> etamake(\", q, 20)
eta(tau)")
```

## Detailed Gap Analysis

### Help System Gaps (help.rs)

**New function help entries needed:**
| Function | Status | Notes |
|----------|--------|-------|
| `radsimp` | MISSING | Identity function for Maple compat |
| `min` | EXISTS | Already has help entry |
| `max` | EXISTS | Already has help entry |
| `subs` | EXISTS | Already has help entry -- may need update for indexed vars |
| `jac2series` | EXISTS | Already has help entry -- may need update for 2-arg form |
| `quinprod` | EXISTS | Already has help entry -- may need update for prodid/seriesid |

**New language construct help entries needed:**
| Feature | Status | Notes |
|---------|--------|-------|
| `ditto` (") | MISSING | New in v4.0, no help entry yet |
| `lambda`/arrow (->) | MISSING | New in v4.0, no help entry yet |
| `RETURN` | EXISTS in `proc` help | Already documented |

**Existing entries needing signature updates:**
| Function | Current Signature | Needed Update |
|----------|-------------------|---------------|
| `aqprod` | `aqprod(a, q, n) or aqprod(a, q, n, T) or aqprod(a, q, infinity, T)` | Already covers 3-arg form correctly |
| `theta3` | `theta3(T) or theta3(q, T) or theta3(a, q, T)` | Already covers 2-arg form |
| `qfactor` | `qfactor(f, q) or qfactor(f, T) or qfactor(f, q, T)` | Already covers 2-arg form |
| `jac2series` | `jac2series(JP, q, T)` | May need update to show 2-arg `jac2series(JP, T)` form |
| `quinprod` | `quinprod(z, q, T)` | May need update to show prodid/seriesid modes |
| `subs` | `subs(var=val, expr)` | May need update for indexed vars `X[1]=val` |

**general_help() updates needed:**
- Add `min` and `max` to Number Theory section (ALREADY DONE - line 123-124)
- Add `radsimp` somewhere appropriate (currently not listed)
- Add `subs` to Polynomial Operations (ALREADY DONE - line 58)

### Tab Completion Gaps (repl.rs)

**Missing from canonical_function_names():**
| Name | In eval.rs | In repl.rs | Action |
|------|-----------|-----------|--------|
| `min` | YES | NO | ADD |
| `max` | YES | NO | ADD |
| `radsimp` | YES | NO | ADD |
| `read` | YES | NO | ADD (script loading) |

**Already present (no action needed):**
`jac2series`, `quinprod`, `subs` -- all already in the list.

### PDF Manual Gaps

**New chapter needed:** `16-v4-changes.typ` covering all 14 v4.0 requirements.

**Existing chapters needing updates:**
| Chapter | Update Needed |
|---------|---------------|
| `04-expression-language.typ` | Add Value types: FractionalPowerSeries, QProduct, EtaQuotient. Update function count from 97 to new total. Mention ditto operator. |
| `template.typ` | Update version from "0.9.0" to appropriate v4.0 version |
| `main.typ` | Add `#include` for new chapter |

### qmaple.pdf Example Classification

**Section 3: Product Conversion (p.5-11) -- EXECUTABLE**
- Section 3.1 prodmake: Rogers-Ramanujan sum via for-loop, series, prodmake (p.5-6) -- ALL executable
- Section 3.2 qfactor: T(r,j) recursive procedure, factor, qfactor (p.6-8) -- EXECUTABLE (uses proc, for, if, RETURN, floor, qbin, min, expand, factor, qfactor)
- Dixon sum procedure (p.7-8) -- EXECUTABLE (uses proc, for, min, qbin, RETURN, expand, qfactor)
- Section 3.3 etamake: theta2/3/4 eta representations (p.8-9) -- EXECUTABLE (theta2/3/4 with Garvan 2-arg form, etamake)
- Section 3.4 jacprodmake: Rogers-Ramanujan jacprodmake, jac2prod, jac2series (p.10-11) -- EXECUTABLE

**Section 4: Relations (p.12-19) -- MOSTLY EXECUTABLE**
- Section 4.1 findhom: theta function relations (p.12-13) -- EXECUTABLE
- Section 4.2 findhomcombo: Eisenstein series UE procedure (p.13-14) -- EXECUTABLE (uses legendre/L, for, proc)
- Section 4.3 findnonhom: theta quotient relations (p.15-16) -- PARTIALLY EXECUTABLE (uses arrow `->` which is now supported; uses `subs` with indexed vars)
- Section 4.4 findnonhomcombo: eta quotient relations (p.16-17) -- EXECUTABLE
- Section 4.5 findpoly: theta/cubic modular identity (p.17-18) -- EXECUTABLE (uses radsimp, findpoly)

**Section 5: Sifting (p.19-20) -- EXECUTABLE**
- Ramanujan p(5n+4) identity, sift, etamake (p.19-20) -- EXECUTABLE

**Section 6: Product Identities (p.20-25) -- MOSTLY EXECUTABLE**
- Section 6.1 Triple product (p.20-21) -- EXECUTABLE (tripleprod with symbolic z)
- Section 6.2 Quintuple product (p.21-23) -- EXECUTABLE (quinprod with prodid/seriesid modes, sifting application)
- Section 6.3 Winquist (p.23-25) -- EXECUTABLE (winquist with symbolic a/b, tripleprod)

**Not Yet Supported examples:**
- Exercise 4 (p.9): Uses `RootOf(z^2 + z + 1 = 0)` -- requires algebraic numbers (OUT OF SCOPE)
- `with(qseries)` / `with(numtheory)` -- Maple session commands, not needed (no-op in q-Kangaroo)
- Any examples using `while` loops (deferred MAPLE-02)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw struct display for qfactor | Product-form `(1-q^a)(1-q^b)...` | Phase 49 (v4.0) | Help examples can show real output |
| Raw struct for etamake | `eta(k*tau)` notation | Phase 49 (v4.0) | Help examples can show real output |
| jac2series 3-arg only | jac2series 2-arg `(JP, T)` supported | Phase 50 (v4.0) | Help signature needs update |
| No ditto operator | `"` references last result | Phase 47 (v4.0) | Needs new help entry |
| No arrow syntax | `F := q -> expr` for lambdas | Phase 47 (v4.0) | Needs new help entry |

## Open Questions

1. **Help entry for `read`**
   - What we know: `read("file.qk")` is in ALL_FUNCTION_NAMES but has no help entry and is not in tab completion
   - What's unclear: Whether this was intentionally omitted from help
   - Recommendation: Add both help entry and tab completion since it's a real function. Not a v4.0 feature though, so handle quietly.

2. **Version number for template.typ**
   - What we know: Currently "0.9.0". v4.0 is the milestone name but not a version string.
   - What's unclear: What the release version should be
   - Recommendation: Leave at Claude's discretion or ask user. Could be "1.0.0" since v4.0 completes the qmaple.pdf parity goal.

3. **Function count after additions**
   - What we know: Currently 97 functions in FUNC_HELP. Adding `radsimp` makes 98. Tab completion currently 97, adding `min`, `max`, `radsimp`, `read` makes 101.
   - What's unclear: Whether to also count language constructs in the headline count
   - Recommendation: Keep "97 functions" as the FUNC_HELP count (add radsimp = 98), keep language constructs (ditto, lambda) as separate match arms. Update all count assertions.

4. **Whether ditto/lambda should also appear in general_help()**
   - What we know: for/proc/if appear in the Scripting category of general_help()
   - Recommendation: Add ditto under a "Language" section or in the Commands section. Add arrow/lambda under Scripting.

## Sources

### Primary (HIGH confidence)
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/help.rs` -- Complete help system with all 97 entries + 3 language constructs
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/repl.rs` -- Tab completion with 97 function names + 18 keywords + 8 commands
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/eval.rs` -- ALL_FUNCTION_NAMES (definitive list), ALL_ALIAS_NAMES
- `C:/cygwin64/home/Owner/Kangaroo/manual/template.typ` -- Typst macros: func-entry, repl, repl-block, index
- `C:/cygwin64/home/Owner/Kangaroo/manual/main.typ` -- Manual structure, 15 chapter includes
- `C:/cygwin64/home/Owner/Kangaroo/manual/chapters/04b-scripting.typ` -- Pattern for v3.0 chapter with func-entry and repl examples
- `C:/cygwin64/home/Owner/Kangaroo/qmaple.pdf` -- Garvan (1998), 25 pages, all executable examples identified

### Secondary (MEDIUM confidence)
- `C:/cygwin64/home/Owner/Kangaroo/.planning/REQUIREMENTS.md` -- 14 v4.0 requirements
- `C:/cygwin64/home/Owner/Kangaroo/.planning/ROADMAP.md` -- Phase 47-50 details and success criteria

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new tools, all patterns already established
- Architecture: HIGH -- all three surfaces (help, completion, manual) thoroughly examined with exact patterns documented
- Pitfalls: HIGH -- exact count assertions identified, gap analysis complete with specific line numbers
- qmaple.pdf coverage: HIGH -- all 25 pages read, every example classified as executable/translatable/unsupported

**Research date:** 2026-02-21
**Valid until:** Indefinite (documentation patterns are stable)
