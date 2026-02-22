# Phase 53: Lists & List Operations - Research

**Researched:** 2026-02-21
**Domain:** CLI evaluator -- list values, list indexing, list manipulation functions (nops, op, map, sort)
**Confidence:** HIGH

## Summary

Phase 53 adds first-class list support to the q-Kangaroo CLI. The foundation is already partially in place: `AstNode::List` exists in the AST, `Value::List(Vec<Value>)` exists in the Value enum, list literal parsing `[a, b, c]` works, and display formatting (including matrix layout) is implemented. What is **missing** is: (1) list indexing `L[2]` that resolves at runtime against list values, (2) the `nops` function, (3) the `op` function, (4) the `map` function, and (5) the `sort` function.

The critical design challenge is **subscript refactoring**. The parser currently encodes `X[1]` as `Variable("X[1]")` -- a string-munged variable name that gets looked up literally in the environment. This works for the existing table-style indexed variable pattern (used in Phase 52 for `X[1] := 5`) but fundamentally cannot support list indexing, because `L[2]` must look up `L` as a list value and then extract element 2, not look up a variable literally named `"L[2]"`. The parser needs a new `AstNode::Index { expr, index }` variant, and the evaluator must handle it by evaluating `expr`, checking if the result is a `Value::List`, and then indexing into it.

**Primary recommendation:** Add `AstNode::Index` to the AST, refactor the parser's subscript handling to emit it, update the evaluator to handle list indexing (1-indexed, Maple convention), and add the four new functions (nops, op, map, sort) to the dispatch table. Maintain backward compatibility with `X[1] := 5` by teaching the Assign handler to recognize `AstNode::Index` on the left-hand side.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LANG-02 | List literals `[a, b, c]` as first-class values with display and indexing `L[i]` | List literals already parse and evaluate. Indexing requires new AstNode::Index variant + evaluator support. Display already works via format_list(). |
| LIST-01 | `nops(expr)` returns the number of operands/elements | New dispatch entry. For lists: return length. For series: return number of nonzero terms. For integers/rationals: return 1. |
| LIST-02 | `op(i, expr)` extracts the i-th operand/element | New dispatch entry. For lists: 1-indexed element access. For series: return i-th nonzero term as pair or coefficient. |
| LIST-03 | `map(f, list)` applies function f to each element | New dispatch entry. First arg is procedure or symbol (builtin name). Applies to each list element, returns new list. |
| LIST-04 | `sort(list)` sorts list elements | New dispatch entry. Numeric ordering for integers/rationals. Lexicographic for symbols/strings. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Existing AST/parser | N/A | `AstNode::Index` variant, parser subscript refactor | Already have Pratt parser with subscript handling |
| Existing eval.rs | N/A | Dispatch for nops/op/map/sort, eval_expr for Index | All 101 functions follow same dispatch pattern |
| Existing format.rs | N/A | List display `[1, 2, 3]` already implemented | No changes needed for display |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rug::Integer | existing | QInt comparison for sort | Already in deps |
| std::cmp::Ordering | stdlib | Sorting comparisons | Already imported in format.rs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| AstNode::Index { expr, index } | Keep Variable("X[1]") hack | Cannot support runtime list indexing; would require environment-level heuristic that checks if base var is a list |

## Architecture Patterns

### Current File Structure (no new files needed)
```
crates/qsym-cli/src/
  ast.rs           # Add AstNode::Index variant
  parser.rs        # Refactor subscript handling to emit Index
  eval.rs          # Handle Index in eval_expr, add nops/op/map/sort dispatch
  help.rs          # Add help entries for new functions
  repl.rs          # Add nops/op/map/sort to canonical_function_names()
  format.rs        # No changes needed (list display already works)
  environment.rs   # No changes needed
```

### Pattern 1: AstNode::Index Variant
**What:** New AST node for subscript access: `expr[index_expr]`
**When to use:** Any `X[i]` syntax in the parser
**Example:**
```rust
// In ast.rs:
/// Subscript/index access: `expr[index]`.
Index {
    expr: Box<AstNode>,
    index: Box<AstNode>,
},
```

### Pattern 2: Parser Subscript Refactoring
**What:** Replace the current `Variable("X[1]")` string-munging with `AstNode::Index`
**When to use:** The LED (infix) section of `expr_bp` when `Token::LBracket` is encountered
**Critical detail:** Must work for ANY lhs expression (not just Variable), and must allow arbitrary index expressions (not just integer literals). The current restriction to integer-only subscripts is unnecessary.
**Example:**
```rust
// In parser.rs, the LBracket LED section:
if *self.peek() == Token::LBracket {
    if 19 < min_bp { break; }
    self.advance(); // consume [
    let index = self.expr_bp(0)?;
    self.expect(&Token::RBracket, "']' to close subscript")?;
    lhs = AstNode::Index {
        expr: Box::new(lhs),
        index: Box::new(index),
    };
    continue;
}
```

### Pattern 3: Evaluator Index Handling
**What:** eval_expr handles AstNode::Index by evaluating the base expression, checking for Value::List, and extracting the element
**Key detail:** 1-indexed (Maple convention). Out-of-range produces a clear error.
**Example:**
```rust
// In eval.rs eval_expr:
AstNode::Index { expr, index } => {
    let base = eval_expr(expr, env)?;
    let idx_val = eval_expr(index, env)?;
    let i = match idx_val {
        Value::Integer(n) => n.0.to_i64().ok_or_else(|| EvalError::Other(
            "index too large".to_string()
        ))?,
        _ => return Err(EvalError::Other(
            format!("index must be an integer, got {}", idx_val.type_name())
        )),
    };
    match base {
        Value::List(items) => {
            // Maple uses 1-indexing
            if i < 1 || i as usize > items.len() {
                return Err(EvalError::Other(format!(
                    "list index {} out of range (list has {} elements)",
                    i, items.len()
                )));
            }
            Ok(items[(i - 1) as usize].clone())
        }
        _ => Err(EvalError::Other(format!(
            "cannot index into {}", base.type_name()
        ))),
    }
}
```

### Pattern 4: Indexed Assignment Compatibility
**What:** `L[2] := 5` must continue to work, and ideally should mutate lists in-place
**Critical design decision:** The current parser treats `X[1] := 5` by parsing `X[1]` as `Variable("X[1]")` and then handling assign on that variable name. After the refactor, `X[1]` parses as `AstNode::Index { expr: Variable("X"), index: Integer(1) }`. The Assign handler in the parser only accepts `AstNode::Variable` on the lhs. Two options:

**Option A (Recommended -- simpler):** Keep the parser's Assign handler as-is for `Variable("name") := expr`. For `Index { expr: Variable("L"), index }`, recognize this pattern in the Assign LED or create a new `AstNode::IndexAssign` variant. Evaluate the index, check if L is a list, and mutate element in place.

**Option B:** In the parser, when we see `Index` followed by `:=`, produce a special `AstNode::IndexAssign { name, index, value }`.

**Backward compatibility:** The existing `X[1] := 5` pattern (used for table-style indexed variables in Phase 52) stored values under the literal key `"X[1]"`. After this refactor, `X[1] := 5` would attempt to index into `X` instead. This is the CORRECT Maple behavior -- in Maple, `X[1] := 5` means "set the first element of X to 5". If X doesn't exist yet as a list, we should treat this as creating a table entry (fall back to the `Variable("X[1]")` pattern), OR we could simply require X to already be a list. Given that Phase 52 may have introduced `X[1] := 5` tests, we need to handle this carefully.

**Safest approach:** In the evaluator, when processing `Index { expr: Variable("X"), index: Integer(i) }`, first check if `X` is bound to a `Value::List`. If so, do list element mutation. If `X` is unbound or not a list, fall back to the old behavior of setting `"X[i]"` as a variable name. This maintains full backward compatibility.

### Pattern 5: Function Dispatch Pattern
**What:** All four new functions follow the established dispatch pattern in eval.rs
**Example:**
```rust
// In dispatch():
"nops" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::List(items) => Ok(Value::Integer(QInt::from(items.len() as i64))),
        Value::Series(fps) => {
            let count = fps.iter().filter(|(_, c)| !c.is_zero()).count();
            Ok(Value::Integer(QInt::from(count as i64)))
        },
        Value::Integer(_) | Value::Rational(_) => Ok(Value::Integer(QInt::from(1i64))),
        _ => Err(EvalError::ArgType { ... }),
    }
}
```

### Anti-Patterns to Avoid
- **String-munging for subscripts:** Do NOT continue the `Variable("X[1]")` pattern for new list features. This was a stopgap for table-style access and does not support runtime list indexing.
- **Hardcoded function names in map:** When implementing `map(f, list)`, do NOT hardcode specific function names. Use the existing dispatch mechanism -- if `f` is a symbol, look it up as a builtin; if `f` is a procedure, call it via call_procedure.
- **0-indexed lists:** Maple uses 1-indexing. The success criteria explicitly states `L[2]` returns the second element. Do NOT use 0-indexing.
- **Mutating sort:** `sort` should return a NEW sorted list, not mutate in place. Maple's `sort` returns a new list.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| List display | Custom display for lists | Existing `format_list()` in format.rs | Already handles `[1, 2, 3]` and matrix layout |
| Function fuzzy matching | New matching for nops/op/map/sort | Existing `find_similar_names()` + `ALL_FUNCTION_NAMES` | Just add names to the array |
| Procedure calling in map | Custom eval logic for map callbacks | Existing `call_procedure()` + `dispatch()` | Both already handle all function call semantics |

## Common Pitfalls

### Pitfall 1: Parser Subscript/Assign Interaction
**What goes wrong:** After refactoring subscript to `AstNode::Index`, the parser's Assign handler (`if let AstNode::Variable(name) = lhs`) no longer matches `L[2] := 5` because lhs is now `AstNode::Index`, not `AstNode::Variable`.
**Why it happens:** The Assign handler has a strict pattern match on `AstNode::Variable`.
**How to avoid:** Add a second arm in the Assign handling that matches `AstNode::Index`. When `Index { expr: Variable(name), index } := value` is seen, produce `AstNode::IndexAssign { name, index, value }`.
**Warning signs:** Tests for `X[1] := 5` fail after parser refactor.

### Pitfall 2: Off-by-One Indexing
**What goes wrong:** Using 0-indexing instead of Maple's 1-indexing.
**Why it happens:** Rust's Vec is 0-indexed; natural to write `items[i]` instead of `items[i-1]`.
**How to avoid:** Always subtract 1 when converting Maple index to Rust index. Add explicit test: `L[1]` returns first element, `L[0]` is an error.
**Warning signs:** `L := [10, 20, 30]; L[1]` returns 20 instead of 10.

### Pitfall 3: map() with Builtin Functions
**What goes wrong:** `map(f, [1,2,3])` where `f` is a builtin like `numbpart` fails because `f` evaluates to `Value::Symbol("f")` or the user passes a string.
**Why it happens:** The first argument to `map` could be a procedure (Value::Procedure), a lambda, or a symbol naming a builtin.
**How to avoid:** In map's implementation: (1) If first arg is `Value::Procedure`, use `call_procedure`. (2) If first arg is `Value::Symbol(name)`, try `dispatch(name, &[elem], env)` for each element. (3) Optionally, allow `Value::String(name)` too.
**Warning signs:** `map(numbpart, [1,2,3,4,5])` returns error instead of `[1, 2, 3, 5, 7]`.

### Pitfall 4: Sort Comparison Across Types
**What goes wrong:** `sort([3, 1/2, 2])` fails because comparing Integer vs Rational.
**Why it happens:** The sorting comparison function doesn't handle mixed numeric types.
**How to avoid:** Convert all numeric types to QRat for comparison. For non-numeric (Symbol, String), use lexicographic ordering as a separate category. Error on truly incomparable types (e.g., mixing numbers and strings in the same list).
**Warning signs:** `sort([3, 1, 2])` works but `sort([3, 1/2, 2])` crashes.

### Pitfall 5: Backward Compatibility with Variable("X[1]")
**What goes wrong:** Existing tests that use `X[1] := 5; X[1]` break because the parser now produces `AstNode::Index` instead of `Variable("X[1]")`.
**Why it happens:** The old pattern stored values under the literal key `"X[1]"` in the environment HashMap.
**How to avoid:** In the evaluator's Index handler, when base is not a list (e.g., unbound variable becomes Symbol), fall back to looking up `"X[1]"` in the environment. Or in the IndexAssign handler, if X is not bound as a list, create the key `"X[1]"`. This preserves table-style semantics.
**Warning signs:** Phase 52 tests that use indexed variables fail.

### Pitfall 6: Negative Indices
**What goes wrong:** Maple supports negative indices: `L[-1]` returns the last element.
**Why it happens:** Success criteria doesn't mention negative indexing, but it's a natural expectation.
**How to avoid:** For MVP, support positive indices only and return a clear error for non-positive indices. Mention Maple supports `L[-1]` for last element as a future enhancement.
**Warning signs:** Users try `L[-1]` and get a confusing error.

### Pitfall 7: Function Count Constants
**What goes wrong:** After adding 4 new functions, the `canonical_function_count` test in repl.rs and `function_count_verification` test in eval.rs fail.
**Why it happens:** These tests assert exact function counts.
**How to avoid:** Update the count assertion from 101 to 105 (or whatever the new total is) in both test files.
**Warning signs:** `cargo test` shows failing assertion on function count.

## Code Examples

### Adding a dispatch entry (established pattern)
```rust
// Source: crates/qsym-cli/src/eval.rs (existing pattern from floor, legendre, etc.)
"nops" => {
    expect_args(name, args, 1)?;
    match &args[0] {
        Value::List(items) => Ok(Value::Integer(QInt::from(items.len() as i64))),
        Value::Series(fps) => {
            // Count nonzero terms
            let count = fps.iter().filter(|(_, c)| !c.is_zero()).count();
            Ok(Value::Integer(QInt::from(count as i64)))
        }
        Value::Integer(_) | Value::Rational(_) => Ok(Value::Integer(QInt::from(1i64))),
        Value::Symbol(_) => Ok(Value::Integer(QInt::from(1i64))),
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 0,
            expected: "list, series, integer, or rational",
            got: other.type_name().to_string(),
        }),
    }
}
```

### Adding to ALL_FUNCTION_NAMES (established pattern)
```rust
// Source: crates/qsym-cli/src/eval.rs (near line 6043)
// Add after existing entries:
// Pattern U: List operations
"nops", "op", "map", "sort",
```

### Adding to canonical_function_names (established pattern)
```rust
// Source: crates/qsym-cli/src/repl.rs (near line 117)
// Add new group:
// Group L: List operations (4)
"nops", "op", "map", "sort",
```

### map implementation with procedure/builtin dispatch
```rust
"map" => {
    expect_args(name, args, 2)?;
    let func = &args[0];
    let list = match &args[1] {
        Value::List(items) => items,
        other => return Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 1,
            expected: "list",
            got: other.type_name().to_string(),
        }),
    };
    let mut result = Vec::with_capacity(list.len());
    for elem in list {
        let val = match func {
            Value::Procedure(proc) => call_procedure(proc, &[elem.clone()], env)?,
            Value::Symbol(fname) => dispatch(fname, &[elem.clone()], env)?,
            other => return Err(EvalError::ArgType {
                function: name.to_string(),
                arg_index: 0,
                expected: "procedure or function name",
                got: other.type_name().to_string(),
            }),
        };
        result.push(val);
    }
    Ok(Value::List(result))
}
```

### sort implementation with numeric ordering
```rust
"sort" => {
    expect_args(name, args, 1)?;
    let list = match &args[0] {
        Value::List(items) => items.clone(),
        other => return Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: 0,
            expected: "list",
            got: other.type_name().to_string(),
        }),
    };
    let mut sorted = list;
    // Try numeric sort first
    sorted.sort_by(|a, b| {
        compare_values_for_sort(a, b).unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(Value::List(sorted))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Variable("X[1]")` string hack | `AstNode::Index { expr, index }` | This phase | Enables runtime list indexing; backward compat via evaluator fallback |
| No list manipulation functions | nops/op/map/sort in dispatch | This phase | Maple-compatible list operations |

**Existing infrastructure (no changes needed):**
- `AstNode::List(Vec<AstNode>)` -- already in ast.rs
- `Value::List(Vec<Value>)` -- already in eval.rs
- `format_list()` -- already in format.rs, handles `[1, 2, 3]` and matrix layout
- `format_latex` for lists -- already handles `\left[...\right]`
- List literal parsing `[expr, expr, ...]` -- already in parser.rs
- List evaluation (eval each element) -- already in eval_expr

## Open Questions

1. **Negative indexing (L[-1] for last element)**
   - What we know: Maple supports negative indices for end-relative access
   - What's unclear: Whether the success criteria requires it (it does not mention negative indices)
   - Recommendation: Omit for MVP, return error for non-positive indices. Easy to add later.

2. **op() semantics for non-list types**
   - What we know: In Maple, `op(0, expr)` returns the type, `op(i, expr)` returns the i-th operand of the internal representation. For series, `nops` returns the number of terms.
   - What's unclear: Exact behavior for q-Kangaroo's Value types (BivariateSeries, QProduct, etc.)
   - Recommendation: Support `op(i, list)` for lists (1-indexed). For series, return the i-th nonzero (exponent, coefficient) pair as a list `[exponent, coefficient]`. For other types, return an error or the value itself if i=1 and nops=1.

3. **map() with multi-argument functions**
   - What we know: Maple's `map` can pass extra arguments: `map(f, L, arg1, arg2)` calls `f(elem, arg1, arg2)` for each element
   - What's unclear: Whether the success criteria requires this (it says `map(f, [1,2,3])` only)
   - Recommendation: Implement basic `map(f, list)` first. Multi-argument map is a stretch goal.

4. **sort() custom comparison**
   - What we know: Maple's `sort(L, F)` accepts a custom comparison function F
   - What's unclear: Whether needed for success criteria (it says `sort([3,1,2])` only)
   - Recommendation: Implement numeric sort only for MVP. Custom comparison is a stretch goal.

5. **Table-style backward compatibility**
   - What we know: Phase 52 may have tests using `X[1] := 5` with the old `Variable("X[1]")` pattern
   - What's unclear: Exact extent of existing tests relying on this pattern
   - Recommendation: Check Phase 52 tests. Implement evaluator-level fallback so `X[1]` when X is not a list falls back to environment lookup of `"X[1]"`.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `crates/qsym-cli/src/ast.rs` -- AstNode::List already exists (line 80)
- Codebase analysis: `crates/qsym-cli/src/eval.rs` -- Value::List already exists (line 71), dispatch pattern (line 3225)
- Codebase analysis: `crates/qsym-cli/src/parser.rs` -- List parsing (line 186-200), subscript handling (line 347-366)
- Codebase analysis: `crates/qsym-cli/src/format.rs` -- format_list (line 269-286), format_latex list (line 995-997)
- Codebase analysis: `crates/qsym-cli/src/repl.rs` -- canonical_function_names (line 66-118)

### Secondary (MEDIUM confidence)
- [Maple nops documentation](https://www.maplesoft.com/support/help/Maple/view.aspx?path=op) -- nops returns number of operands
- [Advanced Maple Functions (RISC)](https://www3.risc.jku.at/education/courses/ws2018/cas/advanced.html) -- op/nops behavior
- [Maple map documentation](https://www.maplesoft.com/support/help/maple/view.aspx?path=map) -- map applies function to each operand
- [Maple sort documentation](https://www.maplesoft.com/support/help/maple/view.aspx?path=sort) -- sort with numeric/lexicographic ordering

### Tertiary (LOW confidence)
- Maple negative indexing semantics -- inferred from general Maple knowledge, not verified against current docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All code is in the existing crate, no new dependencies
- Architecture (AstNode::Index): HIGH - Straightforward Pratt parser extension, well-understood pattern
- Architecture (dispatch): HIGH - Follows exact same pattern as all 101 existing functions
- Pitfalls: HIGH - Identified from direct codebase analysis of existing subscript handling
- Maple semantics (nops/op): MEDIUM - Based on web search and general Maple knowledge; exact edge cases for non-list types may vary

**Research date:** 2026-02-21
**Valid until:** 2026-03-21 (stable -- this is internal Rust code, no external API changes)
