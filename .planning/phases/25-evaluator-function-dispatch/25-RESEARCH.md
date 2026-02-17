# Phase 25: Evaluator & Function Dispatch - Research

**Researched:** 2026-02-17
**Domain:** AST evaluation, function dispatch, variable environment, text output formatting
**Confidence:** HIGH

## Summary

Phase 25 connects the Phase 24 parser (AST) to qsym-core's 79 functions via an evaluator that walks AstNode trees, dispatches function calls, manages a variable environment, and formats results as text. The evaluator lives in `crates/qsym-cli/src/` alongside the existing parser, lexer, AST, and token modules.

The core challenge is bridging two type worlds: the parser's dynamically-typed AstNode (integers, series, lists, product forms, relation vectors) and qsym-core's statically-typed Rust functions. The evaluator needs a `Value` enum that unifies all possible return types, a dispatch table mapping function names to handler closures, and an environment (HashMap) for variable storage.

**Primary recommendation:** Build a `Value` enum with variants for all return types (Series, Integer, Rational, Dict, List, Optional, Pair, None), a `dispatch(name, args) -> Result<Value, EvalError>` function with a static match table for all 79 functions plus Maple aliases, and wrap each evaluation in `std::panic::catch_unwind` with `AssertUnwindSafe` for robustness.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Wrong argument count errors MUST show expected signature: "Error: aqprod expects 4 arguments (a, q, n, N), got 2"
- REPL MUST catch panics from qsym-core (e.g., division by zero) and continue session -- never crashes
- Unknown function errors MUST suggest similar names: "Error: unknown function 'etaq2'. Did you mean: etaq, theta2?"
- Variables and inline expressions are interchangeable -- evaluator resolves either
- List arguments use bracket syntax `[...]` -- requires parser extension for `[...]` list literals
- Non-series results use Maple-style formatting

### Claude's Discretion
- Case-sensitivity of Maple aliases (RECOMMEND: case-insensitive lookup via `.to_lowercase()`)
- Scope and content of alias table (RECOMMEND: all Garvan names where they differ from q-Kangaroo -- see alias table below)
- Alias notification behavior (RECOMMEND: silent -- just works, no notice printed)
- Truncation parameter handling (RECOMMEND: optional last parameter with session default of 20)
- Series display term count (RECOMMEND: show all computed terms, rely on FPS Display impl which already does this)
- Integer result format (RECOMMEND: just the number, no label)
- Product form display style (RECOMMEND: `prod(1-q^i)^e_i` notation with factors sorted by i)
- Runtime error detail level (RECOMMEND: user-friendly message with function name and cause, no stack trace)
- Session parameter visibility (RECOMMEND: implicit -- user never sees session, evaluator injects it)
- Hypergeometric argument syntax (RECOMMEND: bracket list of bracket tuples `[(1,1,2), (1,1,3)]` for num/den/pow params)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| qsym-core | local | All 79 mathematical functions | This project's computation engine |
| std::panic | stdlib | catch_unwind for panic recovery | Only stable Rust mechanism for catching panics |
| std::collections::HashMap | stdlib | Variable environment | Standard associative container |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| No external crates needed | -- | -- | The evaluator is pure Rust glue code between parser AST and qsym-core |

**Installation:**
No new dependencies. The `qsym-cli` crate already depends on `qsym-core`.

## Architecture Patterns

### Recommended Project Structure
```
crates/qsym-cli/src/
  ast.rs          # (Phase 24 -- unchanged)
  error.rs        # (Phase 24 -- unchanged)
  lexer.rs        # (Phase 24 -- extended for [...])
  parser.rs       # (Phase 24 -- extended for [...])
  token.rs        # (Phase 24 -- extended for LBracket, RBracket)
  lib.rs          # (Phase 24 -- add eval module)
  main.rs         # (Phase 26 -- REPL loop, out of scope)
  eval.rs         # NEW: evaluator core (Value enum, eval_stmt, eval_expr)
  dispatch.rs     # NEW: function dispatch table (all 79 functions + aliases)
  format.rs       # NEW: output formatting for all Value types
  environment.rs  # NEW: variable environment + session state
```

### Pattern 1: Value Enum (Tagged Union for Return Types)
**What:** A single enum that unifies all possible evaluation results.
**When to use:** Every function call returns a Value, every variable stores a Value.
**Example:**
```rust
use qsym_core::number::{QInt, QRat};
use qsym_core::series::FormalPowerSeries;

/// Runtime value in the evaluator.
#[derive(Clone, Debug)]
pub enum Value {
    /// FormalPowerSeries (most common return type)
    Series(FormalPowerSeries),
    /// Exact integer (partition_count, findmaxind indices)
    Integer(QInt),
    /// Exact rational number
    Rational(QRat),
    /// List of values (findlincombo coefficients, findmaxind indices, etc.)
    List(Vec<Value>),
    /// Key-value map (prodmake, etamake, qfactor, findpoly results)
    Dict(Vec<(String, Value)>),
    /// Pair of values (heine1/2/3 return (prefactor, result))
    Pair(Box<Value>, Box<Value>),
    /// Boolean value
    Bool(bool),
    /// None/null (try_summation returns None on failure)
    None,
}
```

### Pattern 2: Function Dispatch via Match Table
**What:** A single `dispatch(name, args, env) -> Result<Value, EvalError>` function with a large match on lowercase function name.
**When to use:** Every FuncCall AstNode triggers dispatch.
**Example:**
```rust
pub fn dispatch(
    name: &str,
    args: &[Value],
    env: &mut Environment,
) -> Result<Value, EvalError> {
    // Case-insensitive lookup for Maple aliases
    let canonical = resolve_alias(&name.to_lowercase());

    match canonical.as_str() {
        "aqprod" => dispatch_aqprod(args, env),
        "etaq" => dispatch_etaq(args, env),
        "partition_count" => dispatch_partition_count(args),
        // ... all 79 functions ...
        _ => {
            let suggestions = find_similar_names(canonical.as_str());
            Err(EvalError::UnknownFunction {
                name: name.to_string(),
                suggestions,
            })
        }
    }
}
```

### Pattern 3: Argument Extraction Helpers
**What:** Type-safe helpers to extract i64, QRat, FPS, Vec<FPS>, etc. from Value slices.
**When to use:** Every dispatch function uses these to validate and extract arguments.
**Example:**
```rust
fn expect_args(name: &str, args: &[Value], expected: usize) -> Result<(), EvalError> {
    if args.len() != expected {
        return Err(EvalError::WrongArgCount {
            function: name.to_string(),
            expected,
            got: args.len(),
            // Include signature hint
            signature: get_signature(name),
        });
    }
    Ok(())
}

fn extract_i64(name: &str, args: &[Value], index: usize) -> Result<i64, EvalError> {
    match &args[index] {
        Value::Integer(n) => {
            // Convert QInt to i64 (may fail for huge integers)
            n.0.to_i64().ok_or_else(|| EvalError::ArgType {
                function: name.to_string(),
                arg_index: index,
                expected: "integer (fits in i64)",
                got: "integer too large".to_string(),
            })
        }
        other => Err(EvalError::ArgType {
            function: name.to_string(),
            arg_index: index,
            expected: "integer",
            got: format!("{:?}", value_type_name(other)),
        }),
    }
}

fn extract_series(name: &str, args: &[Value], index: usize) -> Result<&FormalPowerSeries, EvalError> {
    match &args[index] {
        Value::Series(fps) => Ok(fps),
        other => Err(EvalError::ArgType { /* ... */ }),
    }
}

fn extract_series_list(name: &str, args: &[Value], index: usize) -> Result<Vec<&FormalPowerSeries>, EvalError> {
    match &args[index] {
        Value::List(items) => {
            items.iter().enumerate().map(|(i, v)| {
                match v {
                    Value::Series(fps) => Ok(fps),
                    _ => Err(EvalError::ArgType { /* list element not a series */ }),
                }
            }).collect()
        }
        other => Err(EvalError::ArgType { /* not a list */ }),
    }
}
```

### Pattern 4: Panic Catching per Evaluation
**What:** Wrap each `eval_stmt` in `catch_unwind` with `AssertUnwindSafe`.
**When to use:** At the top-level evaluation loop (one catch per statement, not per function).
**Example:**
```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

pub fn eval_stmt_safe(
    stmt: &Stmt,
    env: &mut Environment,
) -> Result<Option<Value>, EvalError> {
    // AssertUnwindSafe is needed because Environment contains rug types
    // which don't implement UnwindSafe. This is safe because we don't
    // observe partial state after a panic -- the environment is intact.
    let result = catch_unwind(AssertUnwindSafe(|| {
        eval_stmt(stmt, env)
    }));

    match result {
        Ok(inner) => inner,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "internal computation error".to_string()
            };
            Err(EvalError::Panic(msg))
        }
    }
}
```

### Pattern 5: Environment with Session State
**What:** The environment holds variables, last result, and a shared SymbolId for q.
**When to use:** Created once at REPL start, persists across lines.
**Example:**
```rust
use qsym_core::symbol::{SymbolId, SymbolRegistry};

pub struct Environment {
    /// User-defined variables
    pub variables: HashMap<String, Value>,
    /// Last computed result (for % reference)
    pub last_result: Option<Value>,
    /// Symbol registry (owns the interning)
    pub symbols: SymbolRegistry,
    /// Cached SymbolId for "q"
    pub sym_q: SymbolId,
    /// Default truncation order
    pub default_order: i64,
}

impl Environment {
    pub fn new() -> Self {
        let mut symbols = SymbolRegistry::new();
        let sym_q = symbols.intern("q");
        Self {
            variables: HashMap::new(),
            last_result: None,
            symbols,
            sym_q,
            default_order: 20,
        }
    }
}
```

### Pattern 6: Parser Extension for List Literals
**What:** Add `LBracket` and `RBracket` tokens, `AstNode::List(Vec<AstNode>)` variant, and parse `[expr, expr, ...]`.
**When to use:** Functions that take list arguments (findlincombo, findhom, findcong moduli, hypergeometric params).
**Example:**
```rust
// In token.rs:
pub enum Token {
    // ... existing variants ...
    LBracket,  // [
    RBracket,  // ]
}

// In ast.rs:
pub enum AstNode {
    // ... existing variants ...
    /// List literal: `[expr1, expr2, ...]`
    List(Vec<AstNode>),
}

// In parser.rs, add to prefix/NUD:
Token::LBracket => {
    self.advance();
    let items = self.parse_bracket_list()?;
    self.expect(&Token::RBracket, "']' to close list")?;
    items  // AstNode::List(items)
}
```

### Anti-Patterns to Avoid
- **Dynamic dispatch with trait objects:** Don't use `Box<dyn Fn>` for the dispatch table. A flat `match` on function name is faster, simpler, and allows different argument patterns per function. The dispatch table is static (79 entries) and never changes at runtime.
- **Separate evaluator per return type:** Don't split evaluation by expected return type. Functions decide their own return type internally. The caller works with the unified `Value` enum.
- **Cloning FPS on every operation:** FPS are large (BTreeMap). Use references where possible. Only clone when storing in the environment or returning from arithmetic operations.
- **Global mutable state:** Don't use `static mut` or `lazy_static` for the session. Pass `&mut Environment` explicitly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Series arithmetic | Custom FPS add/mul/sub | `qsym_core::series::arithmetic::{add,sub,mul,negate,scalar_mul,invert}` | Handles truncation correctly |
| Series display | Custom formatter | `FormalPowerSeries::fmt` (Display impl) | Already formats as `1 - q + 2*q^3 + O(q^20)` |
| Integer display | Custom formatter | `QInt::fmt` and `QRat::fmt` (Display impls) | Already handles arbitrary precision |
| Fuzzy matching | Levenshtein from scratch | Simple edit-distance function (10 lines) or substring matching | Only needs "close enough" for error messages |
| Panic recovery | Custom signal handlers | `std::panic::catch_unwind` | Standard library mechanism |

**Key insight:** qsym-core already has Display implementations for FPS, QInt, and QRat. The evaluator's formatting job is mostly to handle the structured return types (Dict, List, Pair) -- the series/number formatting is already done.

## Common Pitfalls

### Pitfall 1: rug Types and UnwindSafe
**What goes wrong:** `rug::Integer` and `rug::Rational` (behind QInt/QRat) do NOT implement `UnwindSafe`. This means `catch_unwind` with a closure that captures `&mut Environment` won't compile.
**Why it happens:** GMP-based types use raw pointers internally.
**How to avoid:** Wrap the closure in `AssertUnwindSafe`. This is safe because after a panic, the environment's variables may have partial updates but the rug heap is not corrupted (rug uses C-level allocation that survives Rust unwinding). The evaluator does not observe partial state from panicking operations -- each statement either succeeds completely or the result is discarded.
**Warning signs:** Compiler error `the type &mut Environment does not fulfill the required lifetime` when using `catch_unwind`.

### Pitfall 2: Session/SymbolId Mismatch
**What goes wrong:** qsym-core functions need a `SymbolId` for the variable `q`. The Python API uses `QSession` with `get_or_create_symbol_id("q")`. The CLI evaluator must do the same thing without QSession.
**Why it happens:** SymbolId is interned per SymbolRegistry instance. Creating a second registry gives a different SymbolId.
**How to avoid:** Create ONE `SymbolRegistry` in `Environment::new()`, intern "q" once, cache the `SymbolId`, and pass it to every qsym-core function call.
**Warning signs:** Series from different function calls can't be added/multiplied because they have different `variable` fields.

### Pitfall 3: Integer Overflow in Argument Extraction
**What goes wrong:** User passes a BigInteger (`AstNode::BigInteger(String)`) where qsym-core expects `i64`. For example, `partition_count(99999999999999999999999)`.
**Why it happens:** The parser correctly distinguishes `Integer(i64)` from `BigInteger(String)`, but most qsym-core functions take `i64` parameters.
**How to avoid:** For functions that take `i64` parameters, check if the argument is a BigInteger and return a clear error: "Error: argument too large for this function (max i64)". For `partition_count`, which takes `i64`, this is the right behavior. The user should see a descriptive error, not a panic.
**Warning signs:** Silent truncation or panic on large integer arguments.

### Pitfall 4: Infinity Handling
**What goes wrong:** `AstNode::Infinity` appears as a function argument (e.g., `aqprod(q,q,infinity,20)`) but qsym-core's `aqprod` expects `PochhammerOrder::Infinite` (a special enum), not an i64 value.
**Why it happens:** Only `aqprod` uses infinity as an argument. It must be special-cased.
**How to avoid:** In `dispatch_aqprod`, check if the 3rd argument is `Value::Infinity` (or a special sentinel) and map it to `PochhammerOrder::Infinite`. All other functions that take integer arguments should reject infinity with a clear error.
**Warning signs:** Treating infinity as a large integer causes very long computation or overflow.

### Pitfall 5: Hypergeometric Tuple Lists
**What goes wrong:** `phi` and `psi` take lists of `(num, den, power)` tuples. In the REPL, users type `phi([(1,1,2), (1,1,3)], [(1,1,5)], 1, 1, 1, 20)`. This requires the parser to handle nested structures: a list of tuples.
**Why it happens:** The Python API uses Python's native tuple/list syntax. The CLI must parse `(1,1,2)` inside `[...]`.
**How to avoid:** For the REPL, parse `(expr, expr, expr)` inside list literals as a 3-element sub-list. The evaluator then interprets `[[1,1,2], [1,1,3]]` as a list of monomial specs. Alternatively, since these are always 3-integer tuples, parse `[1,1,2 ; 1,1,3]` or just require flat syntax like `phi([1,1,2, 1,1,3], ...)` with stride-3 unpacking. RECOMMENDATION: Use the nested `[(1,1,2), (1,1,3)]` syntax since it matches the Python API and is unambiguous.
**Warning signs:** Users confused by syntax mismatch between Python docs and REPL.

### Pitfall 6: Argument Count Varies by Overloading
**What goes wrong:** Some functions have optional parameters. For example, `aqprod` can be called with 4 args (a, q, n_or_inf, N) in the Maple convention but the qsym-core API takes `(coeff_num, coeff_den, power, n, order)` = 5 required params. The REPL convention must be designed carefully.
**Why it happens:** The Python API decomposes monomials into (num, den, power) triples, but REPL users type `aqprod(q, q, infinity, 20)` where `q` means "the monomial q^1 with coefficient 1".
**How to avoid:** Define a REPL-specific calling convention for each function. For `aqprod`, the REPL convention should be `aqprod(a, q, n_or_infinity, order)` where `a` and `q` are expressions that evaluate to monomials (or integers). The evaluator extracts the QMonomial components internally. Detailed conventions are in the Function Catalog below.
**Warning signs:** Users need to type 6 integers where Maple uses 3 arguments.

## Function Catalog (All 79 Functions)

### Evaluator Return Types

| Return Type | Value Variant | Functions |
|-------------|--------------|-----------|
| FPS (series) | `Value::Series` | aqprod, qbin, etaq, jacprod, tripleprod, quinprod, winquist, theta2/3/4, partition_gf, distinct_parts_gf, odd_parts_gf, bounded_parts_gf, rank_gf, crank_gf, sift, phi, psi, try_summation (if Some), heine1/2/3 (pair), all 20 mock_theta_*, appell_lerch_m, g2, g3, bailey_weak_lemma (pair) |
| Integer | `Value::Integer` | partition_count, qdegree (if Some), lqdegree (if Some) |
| Dict | `Value::Dict` | qfactor, prodmake, etamake, jacprodmake, qetamake, mprodmake, findpoly, findcong, prove_eta_id, bailey_apply_lemma, bailey_chain (list of dicts), bailey_discover, q_gosper, q_zeilberger, verify_wz, q_petkovsek, prove_nonterminating, find_transformation_chain, search_identities |
| Coefficient list | `Value::List` | findlincombo, findhom, findnonhom, findhomcombo, findnonhomcombo, findlincombomodp, findhommodp, findhomcombomodp, findprod |
| Index list | `Value::List` | findmaxind |
| Pair of FPS | `Value::Pair` | heine1, heine2, heine3, bailey_weak_lemma |
| Optional | `Value::None` or inner | try_summation, findlincombo, findhomcombo, findnonhomcombo, findlincombomodp, findhomcombomodp, findpoly, qdegree, lqdegree |

### REPL Calling Conventions (Grouped by Pattern)

**Pattern A: Session-implicit series generators** -- user never types session; evaluator injects sym_q.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `aqprod` | `aqprod(coeff_num, coeff_den, power, n_or_infinity, order)` | `qseries::aqprod(&QMonomial::new(QRat::from((cn,cd)), p), sym_q, poch_order, order)` |
| `qbin` | `qbin(n, k, order)` | `qseries::qbin(n, k, sym_q, order)` |
| `etaq` | `etaq(b, t, order)` | `qseries::etaq(b, t, sym_q, order)` |
| `jacprod` | `jacprod(a, b, order)` | `qseries::jacprod(a, b, sym_q, order)` |
| `tripleprod` | `tripleprod(coeff_num, coeff_den, power, order)` | `qseries::tripleprod(&QMonomial, sym_q, order)` |
| `quinprod` | `quinprod(coeff_num, coeff_den, power, order)` | `qseries::quinprod(&QMonomial, sym_q, order)` |
| `winquist` | `winquist(a_cn, a_cd, a_p, b_cn, b_cd, b_p, order)` | `qseries::winquist(&a, &b, sym_q, order)` |
| `theta2` | `theta2(order)` | `qseries::theta2(sym_q, order)` |
| `theta3` | `theta3(order)` | `qseries::theta3(sym_q, order)` |
| `theta4` | `theta4(order)` | `qseries::theta4(sym_q, order)` |
| `partition_gf` | `partition_gf(order)` | `qseries::partition_gf(sym_q, order)` |
| `distinct_parts_gf` | `distinct_parts_gf(order)` | `qseries::distinct_parts_gf(sym_q, order)` |
| `odd_parts_gf` | `odd_parts_gf(order)` | `qseries::odd_parts_gf(sym_q, order)` |
| `bounded_parts_gf` | `bounded_parts_gf(max_part, order)` | `qseries::bounded_parts_gf(max_part, sym_q, order)` |
| `rank_gf` | `rank_gf(z_num, z_den, order)` | `qseries::rank_gf(&QRat, sym_q, order)` |
| `crank_gf` | `crank_gf(z_num, z_den, order)` | `qseries::crank_gf(&QRat, sym_q, order)` |

**Pattern B: No-session functions** -- pure computation, no sym_q needed.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `partition_count` | `partition_count(n)` | `qseries::partition_count(n)` |

**Pattern C: Series-input analysis functions** -- take a series (variable or expression) as first arg.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `sift` | `sift(series, m, j)` | `qseries::sift(&fps, m, j)` |
| `qdegree` | `qdegree(series)` | `qseries::qdegree(&fps)` |
| `lqdegree` | `lqdegree(series)` | `qseries::lqdegree(&fps)` |
| `qfactor` | `qfactor(series)` | `qseries::qfactor(&fps)` |
| `prodmake` | `prodmake(series, max_n)` | `qseries::prodmake(&fps, max_n)` |
| `etamake` | `etamake(series, max_n)` | `qseries::etamake(&fps, max_n)` |
| `jacprodmake` | `jacprodmake(series, max_n)` | `qseries::jacprodmake(&fps, max_n)` |
| `mprodmake` | `mprodmake(series, max_n)` | `qseries::mprodmake(&fps, max_n)` |
| `qetamake` | `qetamake(series, max_n)` | `qseries::qetamake(&fps, max_n)` |

**Pattern D: Target + list of candidates** -- take target series, list of series, and topshift.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `findlincombo` | `findlincombo(target, [candidates...], topshift)` | `qseries::findlincombo(&target, &refs, topshift)` |
| `findhomcombo` | `findhomcombo(target, [candidates...], degree, topshift)` | `qseries::findhomcombo(...)` |
| `findnonhomcombo` | `findnonhomcombo(target, [candidates...], degree, topshift)` | `qseries::findnonhomcombo(...)` |
| `findlincombomodp` | `findlincombomodp(target, [candidates...], p, topshift)` | `qseries::findlincombomodp(...)` |
| `findhomcombomodp` | `findhomcombomodp(target, [candidates...], p, degree, topshift)` | `qseries::findhomcombomodp(...)` |

**Pattern E: List of series** -- take a list of series and parameters.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `findhom` | `findhom([series...], degree, topshift)` | `qseries::findhom(&refs, degree, topshift)` |
| `findnonhom` | `findnonhom([series...], degree, topshift)` | `qseries::findnonhom(...)` |
| `findhommodp` | `findhommodp([series...], p, degree, topshift)` | `qseries::findhommodp(...)` |
| `findmaxind` | `findmaxind([series...], topshift)` | `qseries::findmaxind(&refs, topshift)` |
| `findprod` | `findprod([series...], max_coeff, max_exp)` | `qseries::findprod(&refs, max_coeff, max_exp)` |
| `findcong` | `findcong(series, [moduli...])` | `qseries::findcong(&fps, &moduli)` |

**Pattern F: Two series + parameters** (findpoly)
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `findpoly` | `findpoly(x, y, deg_x, deg_y, topshift)` | `qseries::findpoly(&x, &y, dx, dy, ts)` |

**Pattern G: Hypergeometric with tuple lists**
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `phi` | `phi(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::eval_phi(&series, sym_q, order)` |
| `psi` | `psi(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::eval_psi(&series, sym_q, order)` |
| `try_summation` | `try_summation(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::try_all_summations(...)` |
| `heine1` | `heine1(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::heine_transform_1(...)` |
| `heine2` | `heine2(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::heine_transform_2(...)` |
| `heine3` | `heine3(upper_list, lower_list, z_num, z_den, z_pow, order)` | `qseries::heine_transform_3(...)` |

**Pattern H: Identity proving**
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `prove_eta_id` | `prove_eta_id(lhs_factors, rhs_factors, level)` | `prove_eta_identity(...)` |
| `search_identities` | `search_identities(query, search_type)` | `IdentityDatabase::search_*()` |

**Pattern I: Mock theta / Appell-Lerch** -- session-implicit, 1-2 args.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `mock_theta_f3` | `mock_theta_f3(order)` | `qseries::mock_theta_f3(sym_q, order)` |
| (19 more mock theta) | `mock_theta_XXX(order)` | `qseries::mock_theta_XXX(sym_q, order)` |
| `appell_lerch_m` | `appell_lerch_m(a_pow, z_pow, order)` | `qseries::appell_lerch_m(a_pow, z_pow, sym_q, order)` |
| `universal_mock_theta_g2` | `g2(a_pow, order)` | `qseries::universal_mock_theta_g2(a_pow, sym_q, order)` |
| `universal_mock_theta_g3` | `g3(a_pow, order)` | `qseries::universal_mock_theta_g3(a_pow, sym_q, order)` |

**Pattern J: Bailey machinery** -- tuple parameters.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `bailey_weak_lemma` | `bailey_weak_lemma(pair_name, a_num, a_den, a_pow, max_n, order)` | `weak_bailey_lemma(...)` |
| `bailey_apply_lemma` | `bailey_apply_lemma(pair_name, (a), (b), (c), max_n, order)` | `bailey_lemma(...)` |
| `bailey_chain` | `bailey_chain(pair_name, (a), (b), (c), depth, max_n, order)` | `bailey_chain(...)` |
| `bailey_discover` | `bailey_discover(lhs, rhs, (a), max_depth, order)` | `bailey_discover(...)` |

**Pattern K: Algorithmic summation** -- no session, concrete q values.
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `q_gosper` | `q_gosper(upper, lower, z_n, z_d, z_p, q_n, q_d)` | `qseries::q_gosper(...)` |
| `q_zeilberger` | `q_zeilberger(upper, lower, z_n, z_d, z_p, n, q_n, q_d, max_order)` | `qseries::q_zeilberger(...)` |
| `verify_wz` | `verify_wz(upper, lower, z_n, z_d, z_p, n, q_n, q_d, max_order, max_k)` | `verify_wz_certificate(...)` |
| `q_petkovsek` | `q_petkovsek(coefficients, q_num, q_den)` | `qseries::q_petkovsek(...)` |

**Pattern L: Nonterminating/transformation**
| Function | REPL Signature | qsym-core Call |
|----------|---------------|----------------|
| `prove_nonterminating` | `prove_nonterminating(upper_fixed, n_offset, lower, z_offset, rhs_numer, rhs_denom, q_n, q_d, n_test, max_order)` | `prove_nonterminating(...)` |
| `find_transformation_chain` | `find_transformation_chain(src_upper, src_lower, src_z_n, src_z_d, src_z_p, tgt_upper, tgt_lower, tgt_z_n, tgt_z_d, tgt_z_p, max_depth, order)` | `find_transformation_chain(...)` |

## Maple Alias Table

Case-insensitive lookup. Alias maps to canonical q-Kangaroo name.

| Maple Name | q-Kangaroo Name | Notes |
|------------|----------------|-------|
| `numbpart` | `partition_count` | Garvan: numbpart(n) |
| `rankgf` | `rank_gf` | Garvan: rankgf(z, q, N) |
| `crankgf` | `crank_gf` | Garvan: crankgf(z, q, N) |
| `qphihyper` | `phi` | Garvan: qphihyper([a], [b], q, z, N) |
| `qpsihyper` | `psi` | Garvan: qpsihyper([a], [b], q, z, N) |
| `qgauss` | `try_summation` | Garvan: qgauss(...) |
| `proveid` | `prove_eta_id` | Garvan: proveid(lhs, rhs, level) |
| `qZeil` | `q_zeilberger` | Common abbreviation |
| `qzeilberger` | `q_zeilberger` | Case variation |
| `qPetkovsek` | `q_petkovsek` | Common abbreviation |
| `qpetkovsek` | `q_petkovsek` | Case variation |
| `qgosper` | `q_gosper` | Case variation |
| `findlincombo_modp` | `findlincombomodp` | Garvan underscore convention |
| `findhom_modp` | `findhommodp` | Garvan underscore convention |
| `findhomcombo_modp` | `findhomcombomodp` | Garvan underscore convention |
| `search_id` | `search_identities` | Short form |

**Implementation:** A `HashMap<String, String>` populated at startup. Lookup: `aliases.get(&name.to_lowercase()).unwrap_or(&name)`.

## Output Formatting

### Series (FPS)
Use the existing `Display` impl: `format!("{}", fps)` gives `1 - q - q^2 + q^5 + O(q^20)`.

### Integer
Just the number: `format!("{}", qint)` gives `190569292536040`.

### Rational
Display as fraction: `format!("{}", qrat)` gives `3/7` or `1` (for integers).

### List of Rationals (findlincombo result)
`[1, 0, -3/7]` -- square brackets, comma-separated, rational display.

### List of Integers (findmaxind result)
`[0, 1, 3]` -- square brackets, comma-separated.

### Matrix (findhom result -- list of lists)
```
[[1, 0, -1],
 [0, 1, 2]]
```

### Dict (prodmake, etamake, qfactor, etc.)
```
{factors: {1: -1, 2: -1, 3: -1}, terms_used: 10}
```
For qfactor: `{scalar: 1, factors: {1: 1, 2: 1, 3: 1, 4: 1}, is_exact: true}`

### Pair (heine1/2/3, bailey_weak_lemma)
```
(prefactor, result)
```
where each element is formatted according to its type.

### None (try_summation when no formula applies)
`NONE` or `no result` -- a simple indicator.

### Bool
`true` or `false`.

### Congruences (findcong)
```
[{modulus: 5, residue: 4, divisor: 5},
 {modulus: 7, residue: 5, divisor: 7},
 {modulus: 11, residue: 6, divisor: 11}]
```

## Fuzzy Name Matching

For "Did you mean?" suggestions on unknown functions:

```rust
fn find_similar_names(unknown: &str) -> Vec<String> {
    let all_names = get_all_function_names(); // 79 canonical + aliases
    let mut scored: Vec<(usize, &str)> = all_names
        .iter()
        .filter_map(|name| {
            let dist = edit_distance(unknown, name);
            if dist <= 3 || name.contains(unknown) || unknown.contains(name) {
                Some((dist, name.as_str()))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by_key(|(d, _)| *d);
    scored.into_iter().take(3).map(|(_, n)| n.to_string()).collect()
}

/// Simple Levenshtein edit distance (no external crate needed).
fn edit_distance(a: &str, b: &str) -> usize {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            dp[i+1][j+1] = (dp[i][j] + cost)
                .min(dp[i+1][j] + 1)
                .min(dp[i][j+1] + 1);
        }
    }
    dp[m][n]
}
```

## Expression Evaluation (Arithmetic on Values)

The evaluator must handle `f + g`, `f * g`, `f - g`, `f / g`, `f ^ n`, `-f` where f and g can be any Value types. Core rules:

| Operation | Left | Right | Result | Implementation |
|-----------|------|-------|--------|----------------|
| `+` | Series | Series | Series | `arithmetic::add` |
| `-` | Series | Series | Series | `arithmetic::sub` |
| `*` | Series | Series | Series | `arithmetic::mul` |
| `*` | Integer/Rational | Series | Series | `arithmetic::scalar_mul` |
| `*` | Series | Integer/Rational | Series | `arithmetic::scalar_mul` (flip) |
| `/` | Series | Series | Series | `arithmetic::mul(a, arithmetic::invert(b))` |
| `^` | Series | Integer(n) | Series | Repeated multiplication (or inversion if n<0) |
| `-` (unary) | Series | -- | Series | `arithmetic::negate` |
| `+` | Integer | Integer | Integer | QInt addition |
| `*` | Integer | Integer | Integer | QInt multiplication |
| `/` | Integer | Integer | Rational | QRat division |
| `+` | Rational | Rational | Rational | QRat addition |

Error cases: `Series + Integer` is an error (or could be interpreted as adding a constant series). RECOMMENDATION: For now, reject mixed types with a clear error message.

## SymbolRegistry Access

qsym-core's `SymbolRegistry` is in `qsym_core::symbol`. The evaluator needs direct access:

```rust
use qsym_core::symbol::{SymbolId, SymbolRegistry};

// In Environment::new():
let mut registry = SymbolRegistry::new();
let sym_q = registry.intern("q");
```

Note: `SymbolRegistry::new()` and `intern()` are public. This is NOT the same as the Python API's `SessionInner::get_or_create_symbol_id()` which wraps `arena.symbols_mut().intern()`. For the CLI, we go directly to `SymbolRegistry` since we don't need the full ExprArena.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Python DSL with QSession wrapper | Direct Rust evaluator | Phase 25 | No Python layer, no PyO3 overhead |
| Individual function imports | Single dispatch table | Phase 25 | All 79 functions accessible without explicit registration |

## Open Questions

1. **Series + Integer semantics**
   - What we know: Series and integers are distinct Value types.
   - What's unclear: Should `f + 5` mean `f + 5*q^0` (add constant to series)? Or should it be an error?
   - Recommendation: Support it. Convert integer/rational to a constant FPS before arithmetic. This matches Maple behavior.

2. **Optional truncation order**
   - What we know: The context recommends optional truncation with a session default.
   - What's unclear: How to detect "missing last argument" vs "user explicitly passed a value".
   - Recommendation: For phase 25, require explicit truncation order on all functions. Optional truncation (using session default) can be added as an enhancement.

3. **String arguments in REPL**
   - What we know: `bailey_weak_lemma("rogers-ramanujan", ...)` and `search_identities("classical", "tag")` take string arguments. The current lexer has no string literal token.
   - What's unclear: Whether to add string literals or use bare identifiers.
   - Recommendation: Use bare identifiers for pair names and search terms. The evaluator recognizes specific identifiers as strings for these functions. E.g., `bailey_weak_lemma(rogers_ramanujan, ...)` where `rogers_ramanujan` is treated as the string "rogers-ramanujan" by the dispatch function. Alternatively, add a simple string literal token `"..."` to the lexer.

## Sources

### Primary (HIGH confidence)
- `crates/qsym-python/src/dsl.rs` -- all 79 function signatures, argument types, return types (5417 lines, read in full)
- `crates/qsym-cli/src/ast.rs` -- AstNode enum (185 lines)
- `crates/qsym-cli/src/parser.rs` -- Pratt parser, parse() API (775 lines)
- `crates/qsym-cli/src/token.rs` -- Token enum (126 lines)
- `crates/qsym-core/src/series/display.rs` -- FPS Display impl (87 lines)
- `crates/qsym-core/src/series/arithmetic.rs` -- add, sub, mul, negate, scalar_mul, invert
- `crates/qsym-core/src/series/mod.rs` -- FormalPowerSeries struct definition
- `crates/qsym-core/src/number.rs` -- QInt/QRat Display impls
- `crates/qsym-core/src/lib.rs` -- public module structure
- `docs/examples/maple_migration.ipynb` -- complete Maple-to-q-Kangaroo function mapping (48 cells)

### Secondary (MEDIUM confidence)
- [Rust std::panic::catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) -- panic catching mechanism
- [Rust std::panic::AssertUnwindSafe](https://doc.rust-lang.org/std/panic/struct.AssertUnwindSafe.html) -- unwind safety wrapper
- [Rust std::panic::UnwindSafe](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html) -- unwind safety trait

### Tertiary (LOW confidence)
- rug crate and UnwindSafe: training data suggests rug types don't implement UnwindSafe (not verified with current rug docs, but the compiler will catch it). AssertUnwindSafe is the standard workaround.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all code is in the repo, no external dependencies needed
- Architecture: HIGH -- patterns derived directly from reading dsl.rs (the Python evaluator equivalent)
- Function catalog: HIGH -- every function signature extracted from dsl.rs
- Maple alias table: HIGH -- derived from docs/examples/maple_migration.ipynb
- Pitfalls: HIGH -- identified from actual code structure (rug types, SymbolId, parser limitations)
- Output formatting: MEDIUM -- Display impls verified in code; Dict/List format is recommendation
- Fuzzy matching: HIGH -- simple Levenshtein is well-established, trivial to implement

**Research date:** 2026-02-17
**Valid until:** 2026-03-17 (stable domain, unlikely to change)
