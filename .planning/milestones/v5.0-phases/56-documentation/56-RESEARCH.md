# Phase 56: Documentation - Research

**Researched:** 2026-02-22
**Domain:** Help system, tab completion, and PDF manual for v5.0 features
**Confidence:** HIGH

## Summary

Phase 56 is a documentation-only phase covering all v5.0 features implemented in Phases 52-55. The codebase has well-established patterns from the v4.0 documentation phase (Phase 51) for all three documentation surfaces: (1) help entries in `help.rs` using the `FuncHelp` struct and `function_help()` match arms, (2) tab completion in `repl.rs` via `canonical_function_names()` and `keyword_names`, and (3) PDF manual chapters in Typst using the `#func-entry`, `#repl`, `#repl-block`, and `#index` macros from `template.typ`.

The investigation reveals that Phases 52-55 already added most help entries and tab completion during implementation -- the v5.0 functions (coeff, degree, numer, denom, add, mul, seq, nops, op, map, sort, modp, mods, type, evalb, cat) and the `while` keyword are all already present in help.rs and repl.rs. The gap analysis identifies specific missing items: (1) `print` has no help entry and is not in tab completion despite being a real function, (2) `anames` and `restart` have no help entries despite being dispatchable functions in ALL_FUNCTION_NAMES, (3) the v4.0 manual chapter's "Not Yet Supported" section incorrectly says while loops are unavailable, and (4) the PDF manual needs a new v5.0 chapter (17-v5-changes.typ). The existing `general_help()` text also needs a `print` entry added to the Scripting section.

**Primary recommendation:** Follow the exact same patterns used in Phase 51 -- `FuncHelp` structs for new functions, match arms for special constructs, Typst `#func-entry` for the manual -- but the scope is smaller since most help entries already exist. Focus on filling gaps (print, anames, restart help entries), updating the v4.0 "Not Yet Supported" section, and creating the v5.0 manual chapter.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DOC-01 | Help entries and tab completion for all new functions/keywords | Gap analysis identifies: print() missing help entry and tab completion; anames/restart missing help entries; general_help() needs print added; all v5.0 functions/keywords already have entries from prior phases |
| DOC-02 | PDF manual chapter documenting v5.0 additions | New chapter 17-v5-changes.typ needed; template.typ macros documented; prior v4.0 chapter (16-v4-changes.typ) provides structural pattern; v4.0 "Not Yet Supported" section needs while-loop removal |
</phase_requirements>

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

No new libraries needed. This phase edits existing source files and adds one new Typst chapter.

## Architecture Patterns

### Pattern 1: Function Help Entries (help.rs)

**What:** Each function has a `FuncHelp` struct in the `FUNC_HELP` array. Language constructs (for, while, proc, if, ditto, lambda) use special-case match arms in `function_help()`.

**When to use:** For every new function that needs `?name` help.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/help.rs`

```rust
// For functions: add to FUNC_HELP array
FuncHelp {
    name: "print",
    signature: "print(expr, ...)",
    description: "Display one or more expressions...",
    example: "q> for k from 1 to 3 do print(k) od",
    example_output: "1\n2\n3",
},

// For language constructs: match arm in function_help()
// Already present: "for", "while", "proc", "if", "ditto", "lambda"
```

**Key details:**
- The `FuncHelp` struct has 5 fields: `name`, `signature`, `description`, `example`, `example_output`
- Language construct help uses raw `String::from()` with `\x20` for leading spaces
- The `general_help()` function is a hardcoded string and must be updated separately
- Test `every_canonical_function_has_help_entry` validates a hardcoded list (currently 115 names)
- Test `func_help_count_matches_canonical` asserts `FUNC_HELP.len() == 115`
- `while` already has a match arm in `function_help()` (added in Phase 52)

### Pattern 2: Tab Completion (repl.rs)

**What:** `canonical_function_names()` returns all 117 function names that tab-complete with auto-paren `(`. `keyword_names` lists 15 keywords that complete without paren.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/repl.rs`

```rust
fn canonical_function_names() -> Vec<&'static str> {
    vec![
        // ... 117 entries including all v5.0 functions ...
        // Group U: List Operations (4) -- already present
        "nops", "op", "map", "sort",
        // Group V: Series Coefficients & Utility (9) -- already present
        "coeff", "degree", "numer", "denom", "modp", "mods", "type", "evalb", "cat",
        // Group W: Iteration (3) -- already present
        "add", "mul", "seq",
    ]
}
```

**Key details:**
- Test `canonical_function_count` asserts 117 entries
- Test `no_duplicate_function_names` verifies no duplicates
- Functions complete with trailing `(`, keywords complete without
- `while` is already in `keyword_names` (added in Phase 52)
- `print` is NOT in canonical_function_names -- it is dispatched as a special case in eval.rs before the function lookup, so it was never added

### Pattern 3: PDF Manual Chapters (Typst)

**What:** Each chapter is a separate `.typ` file included from `main.typ`. Uses `#func-entry` for reference entries and `#repl`/`#repl-block` for REPL transcripts.

**Source:** `C:/cygwin64/home/Owner/Kangaroo/manual/`

The v4.0 chapter (`16-v4-changes.typ`) provides the structural pattern:
- Top-level heading: `= What's New in v5.0`
- Sections organized by feature type: Language Features, Bug Fixes, New Functions
- Uses `#func-entry()` for new function reference entries
- Uses `#repl()` and `#repl-block()` for REPL transcript examples
- Uses `#index[]` and `#index-main[]` for index entries
- Includes cross-references to prior decisions (e.g., `cf. Phase 52-01`)

```typst
// In main.typ, add before chapters/15-appendix.typ:
#include "chapters/17-v5-changes.typ"

// New file: chapters/17-v5-changes.typ
#import "../template.typ": *

= What's New in v5.0
#index[v5.0]

== Language Features

=== While Loops
#index[while loops]
// ...

=== print() Function
#index[print function]
// ...
```

### Anti-Patterns to Avoid
- **Don't duplicate help entries that already exist:** All v5.0 functions (coeff, degree, numer, etc.) already have FUNC_HELP entries and tab completion from their implementation phases. Don't re-add them.
- **Don't change existing test counts unnecessarily:** Only update count assertions if you actually add/remove entries from the arrays.
- **Don't forget general_help():** It is a separate hardcoded string from FUNC_HELP. Adding a FuncHelp entry does NOT automatically add it to the general help listing.

## Detailed Gap Analysis

### Help System Gaps (help.rs)

**FUNC_HELP array (currently 115 entries):**

| Function | Has FuncHelp? | Has general_help()? | Action |
|----------|--------------|---------------------|--------|
| `print` | NO | NO | ADD both FuncHelp entry and general_help() listing |
| `anames` | NO | NO | ADD both FuncHelp entry and general_help() listing |
| `restart` | NO | NO | ADD both (restart appears as a command but not as a function) |
| `coeff` | YES | YES | None needed |
| `degree` | YES | YES | None needed |
| `numer` | YES | YES | None needed |
| `denom` | YES | YES | None needed |
| `modp` | YES | YES | None needed |
| `mods` | YES | YES | None needed |
| `type` | YES | YES | None needed |
| `evalb` | YES | YES | None needed |
| `cat` | YES | YES | None needed |
| `nops` | YES | YES | None needed |
| `op` | YES | YES | None needed |
| `map` | YES | YES | None needed |
| `sort` | YES | YES | None needed |
| `add` | YES | YES | None needed |
| `mul` | YES | YES | None needed |
| `seq` | YES | YES | None needed |

**Language construct help (function_help() match arms):**

| Construct | Has match arm? | Action |
|-----------|---------------|--------|
| `while` | YES | None needed (added in Phase 52) |
| `for` | YES | None needed |
| `proc` | YES | None needed |
| `if` | YES | None needed |
| `ditto` | YES | None needed |
| `lambda`/`->` | YES | None needed |

**After adding print, anames, restart:**
- FUNC_HELP count: 115 -> 118
- Test `func_help_count_matches_canonical` must update 115 -> 118
- Test `every_canonical_function_has_help_entry` hardcoded list must add print, anames, restart
- general_help() string must add print, anames, restart entries

### Tab Completion Gaps (repl.rs)

**canonical_function_names() (currently 117 entries):**

| Name | In eval.rs ALL_FUNCTION_NAMES? | In repl.rs? | Action |
|------|-------------------------------|-------------|--------|
| `print` | NO (special-cased before dispatch) | NO | Decision needed: add for tab completion? |
| `anames` | YES | NO | ADD |
| `restart` | YES | NO | ADD |
| All v5.0 funcs | YES | YES | None needed |
| `while` keyword | N/A | YES (keyword_names) | None needed |

Note: `print` is special-cased in eval.rs at line 1194, BEFORE the function dispatch. It is not in ALL_FUNCTION_NAMES. However, it IS a real callable function that users should discover. Recommendation: add it to canonical_function_names() for tab completion.

Note: `anames` and `restart` are both in ALL_FUNCTION_NAMES (eval.rs line 6580) and are callable functions, but were never added to canonical_function_names() in repl.rs. The repl.rs comment says "must match eval.rs ALL_FUNCTION_NAMES exactly" but this has never been strictly true.

**After adding print, anames, restart:**
- canonical_function_names count: 117 -> 120
- Test `canonical_function_count` must update 117 -> 120

### PDF Manual Gaps

**New chapter needed:** `17-v5-changes.typ`

Content outline based on v5.0 features:
1. **Language Features**
   - While loops (syntax, safety limit, REPL multi-line)
   - print() function (intermediate display, returns last value)
   - List literals and indexing ([1,2,3], L[i], 1-indexed)
   - DotDot range syntax (i=1..5 in add/mul/seq)
   - Unicode paste resilience
2. **Bug Fixes**
   - Polynomial division hang fix (POLYNOMIAL_ORDER sentinel)
3. **New Functions**
   - List operations: nops, op, map, sort
   - Series coefficients: coeff, degree
   - Rational decomposition: numer, denom
   - Modular arithmetic: modp, mods
   - Type system: type, evalb
   - String/name operations: cat
   - Iteration: add, mul, seq

**Existing chapter updates needed:**

| Chapter | Update Needed |
|---------|---------------|
| `16-v4-changes.typ` | Remove "while loops" from "Not Yet Supported" section (lines 529-531), since while loops are now implemented |
| `main.typ` | Add `#include "chapters/17-v5-changes.typ"` before appendix |

### Prior Decisions to Document

These design decisions from Phases 52-55 should be noted in the v5.0 manual chapter:

| Decision | Source | Impact on Documentation |
|----------|--------|------------------------|
| print() returns last value (not NULL) | 52-01 | Document return behavior |
| while/for share od-depth counter | 52-02 | Mention in REPL multiline section |
| while no new scope (unlike for) | 52-02 | Document scoping difference |
| 1,000,000 iteration safety limit | 52-02 | Warn in while-loop docs |
| 1-indexed Maple convention | 53-01 | Document in list indexing |
| L[0] is out-of-range error | 53-01 | Document as edge case |
| nops on FPS counts nonzero terms | 53-02 | Note in nops docs |
| op on series returns [exp, coeff] | 53-02 | Document in op docs |
| coeff returns Integer when denom=1 | 54-01 | Document return type |
| type() accepts Symbol and String | 54-01 | Document both forms |
| cat() returns Symbol | 54-01 | Document return type |
| modp/mods correct negative handling | 54-01 | Document in examples |
| DotDot binding power (10,10) | 55-01 | Not user-visible |
| Empty ranges return identity | 55-01 | Document: 0 for add, 1 for mul, [] for seq |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| REPL transcript rendering | Custom Typst blocks | `#repl()` and `#repl-block()` macros from template.typ | Consistent styling |
| Function reference formatting | Manual heading/table layout | `#func-entry()` macro from template.typ | Handles params, examples, related links |
| Index generation | Manual index | `#index[]`, `#index-main[]` from in-dexter 0.7.2 | Automatic back-of-book index |

## Common Pitfalls

### Pitfall 1: Test Count Assertions
**What goes wrong:** Adding entries to `FUNC_HELP` or `canonical_function_names()` without updating the corresponding count assertions causes test failures.
**Why it happens:** Multiple tests assert exact counts:
- `help.rs` test `func_help_count_matches_canonical` asserts `FUNC_HELP.len() == 115`
- `repl.rs` test `canonical_function_count` asserts `canonical_function_names().len() == 117`
- `help.rs` test `every_canonical_function_has_help_entry` checks a hardcoded list of 115 names
- `eval.rs` test `function_count_verification` asserts `>= 85`
**How to avoid:** After adding entries, update ALL count assertions. If adding 3 functions (print, anames, restart): FUNC_HELP 115->118, canonical names 117->120, hardcoded test list += 3.
**Warning signs:** Tests fail with "expected 115, got 118" or "expected 117, got 120".

### Pitfall 2: general_help() Not Updated
**What goes wrong:** Users type `help` and don't see new functions listed.
**Why it happens:** `general_help()` is a hardcoded string, completely separate from `FUNC_HELP`. Adding a `FuncHelp` entry does NOT update the general help listing.
**How to avoid:** After adding any new function, also add its one-liner description to the appropriate category in `general_help()`.

### Pitfall 3: print is NOT in ALL_FUNCTION_NAMES
**What goes wrong:** Assuming print should be in ALL_FUNCTION_NAMES leads to confusion.
**Why it happens:** print() is special-cased BEFORE the function dispatch (eval.rs line 1194), similar to how add/mul/seq are intercepted at line 1214. It never reaches the standard function dispatch.
**How to avoid:** Add print only to canonical_function_names (repl.rs) and FUNC_HELP (help.rs). Do NOT add it to ALL_FUNCTION_NAMES in eval.rs.

### Pitfall 4: v4.0 "Not Yet Supported" Stale Content
**What goes wrong:** The v4.0 chapter says "while loops" are not yet supported, but they are now implemented.
**Why it happens:** 16-v4-changes.typ was written before Phase 52 added while loops.
**How to avoid:** Remove or update the while-loop bullet in the "Not Yet Supported" section of 16-v4-changes.typ (lines 529-531).

### Pitfall 5: Tab Completion vs Function Dispatch Mismatch
**What goes wrong:** `anames` and `restart` are dispatchable functions but don't tab-complete.
**Why it happens:** They were added to ALL_FUNCTION_NAMES in eval.rs but never to canonical_function_names in repl.rs. The repl.rs comment says "must match eval.rs ALL_FUNCTION_NAMES exactly" but this invariant was never enforced.
**How to avoid:** Add both to canonical_function_names. This also means updating the test count from 117.

## Code Examples

### Adding print Help Entry

```rust
// In help.rs FUNC_HELP array (after existing entries, in a new section)
FuncHelp {
    name: "print",
    signature: "print(expr, ...)",
    description: "Display one or more expressions, each on its own line.\n  Useful for showing intermediate results inside loops and procedures.\n  Returns the last argument's value (not NULL, unlike Maple's print).",
    example: "q> for k from 1 to 3 do print(k^2) od",
    example_output: "1\n4\n9",
},
```

### Adding anames Help Entry

```rust
FuncHelp {
    name: "anames",
    signature: "anames()",
    description: "Return a list of all currently assigned variable names.\n  The names are returned as a sorted list of strings.",
    example: "q> x := 1: y := 2: anames()",
    example_output: "[\"x\", \"y\"]",
},
```

### Adding restart Help Entry

```rust
FuncHelp {
    name: "restart",
    signature: "restart()",
    description: "Clear all variables, procedures, and reset the session.\n  Returns the string \"Restart.\"  Equivalent to the clear command.",
    example: "q> x := 42: restart()",
    example_output: "Restart.",
},
```

### Updating general_help() String

```rust
// In the general_help() string, add to the Scripting section:
"
Scripting:
  for            - for-loop: for var from start to end [by step] do body od
  while          - while-loop: while condition do body od
  if             - conditional: if cond then body [elif ...] [else body] fi
  proc           - procedure: name := proc(params) body; end
  RETURN         - early return from procedure: RETURN(value)
  ->             - arrow / lambda: F := x -> expr
  print          - display intermediate values: print(expr, ...)

Variable Management:
  anames         - list all assigned variable names
  restart        - clear all variables and reset the session
"
```

### Manual Chapter Structure (17-v5-changes.typ)

```typst
#import "../template.typ": *

= What's New in v5.0
#index[v5.0]

q-Kangaroo v5.0 closes the remaining Maple language and function gaps.
This chapter documents all features introduced in Phases 52--55.

== Language Features

=== While Loops
#index[while loops]
#index-main[while]

Syntax: `while condition do body od`

#repl-block("q> i := 0: while i < 10 do i := i + 1 od: i
10")

Safety limit: 1,000,000 iterations maximum.

=== The print() Function
#index-main[print]

// ... etc.

== New Functions

=== List Operations

#func-entry(
  name: "nops",
  signature: "nops(expr)",
  description: [...],
  // ...
)

// ... etc.
```

### Updating 16-v4-changes.typ

```typst
// Remove or replace lines 529-531:
// OLD:
// - *`while` loops:* No examples in _qmaple.pdf_ use `while` loops, but
//   this is a general language limitation. Use `for` loops with explicit
//   bounds instead.
//
// NEW: Delete these 3 lines entirely, or replace with:
// - *`while` loops:* Now supported in v5.0.  See @v5-changes.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No while loops | `while cond do body od` | Phase 52 (v5.0) | New language feature, needs manual chapter |
| No print() | `print(expr, ...)` with last-value return | Phase 52 (v5.0) | Needs help entry and manual docs |
| No lists | `[a, b, c]` literals, L[i] indexing | Phase 53 (v5.0) | Major new value type, needs manual chapter |
| No coeff extraction | `coeff(f, q, n)` extracts single coefficient | Phase 54 (v5.0) | Key Maple compat function |
| No range iteration | `add(expr, i=1..5)` / `mul` / `seq` | Phase 55 (v5.0) | Replaces manual for-loops for summation |
| while listed as unsupported | while fully implemented | Phase 52 (v5.0) | v4.0 chapter "Not Yet Supported" is stale |

## Open Questions

1. **Whether to add print to ALL_FUNCTION_NAMES in eval.rs**
   - What we know: print is special-cased before dispatch (line 1194). It works but is not in ALL_FUNCTION_NAMES. anames and restart ARE in ALL_FUNCTION_NAMES.
   - Recommendation: Do NOT add print to ALL_FUNCTION_NAMES. It is intercepted before dispatch like add/mul/seq. Just add it to canonical_function_names in repl.rs and FUNC_HELP in help.rs.

2. **Version string in template.typ**
   - What we know: Currently "0.9.0". Has not been updated since v1.6.
   - Recommendation: Leave as-is for this phase. Version bumps are a release concern, not a documentation concern.

3. **Whether "Variable Management" deserves its own category in general_help()**
   - What we know: anames and restart are currently unlisted. They logically group together. The general_help() already has many categories.
   - Recommendation: Add a "Variable Management:" section between "Scripting:" and "Commands:" in general_help(). Alternatively, add them to the existing "Commands:" section since `restart` and `clear` are session management.

## Sources

### Primary (HIGH confidence)
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/help.rs` -- Complete help system: 115 FUNC_HELP entries + 6 language construct match arms (for, while, proc, if, ditto, lambda)
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/repl.rs` -- Tab completion: 117 canonical function names + 15 keyword names + 8 command names
- `C:/cygwin64/home/Owner/Kangaroo/crates/qsym-cli/src/eval.rs` -- ALL_FUNCTION_NAMES (90 entries), print special-case (line 1194), anames/restart dispatch (lines 4926-4936)
- `C:/cygwin64/home/Owner/Kangaroo/manual/template.typ` -- Typst macros: func-entry, repl, repl-block, index, version "0.9.0"
- `C:/cygwin64/home/Owner/Kangaroo/manual/main.typ` -- Manual structure: 16 chapter includes
- `C:/cygwin64/home/Owner/Kangaroo/manual/chapters/16-v4-changes.typ` -- v4.0 chapter: structural pattern + stale "Not Yet Supported" (while loops)
- `C:/cygwin64/home/Owner/Kangaroo/manual/chapters/04b-scripting.typ` -- v3.0 scripting chapter: pattern for language feature documentation
- `C:/cygwin64/home/Owner/Kangaroo/.planning/milestones/v4.0-phases/51-documentation/51-RESEARCH.md` -- Prior doc phase research: exact same pattern

### Secondary (MEDIUM confidence)
- `C:/cygwin64/home/Owner/Kangaroo/.planning/REQUIREMENTS.md` -- DOC-01 and DOC-02 requirement definitions
- `C:/cygwin64/home/Owner/Kangaroo/.planning/ROADMAP.md` -- Phase 52-55 details, success criteria, prior decisions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new tools, all patterns already established from Phase 51
- Architecture: HIGH -- all three surfaces (help, completion, manual) thoroughly examined with exact line numbers and counts
- Pitfalls: HIGH -- exact count assertions identified, gap analysis complete, stale content found
- Gap analysis: HIGH -- every v5.0 function checked against help.rs, repl.rs, and general_help() individually

**Research date:** 2026-02-22
**Valid until:** Indefinite (documentation patterns are stable)
