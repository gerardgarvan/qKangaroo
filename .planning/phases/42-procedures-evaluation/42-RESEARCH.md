# Phase 42: Procedures & Evaluation - Research

**Researched:** 2026-02-20
**Domain:** Maple-compatible procedure definitions, local variable scoping, memoization, control flow evaluation
**Confidence:** HIGH

## Summary

Phase 42 implements the core of Maple-compatible user-defined procedures and completes the evaluation of control flow AST nodes (ForLoop, IfExpr, Compare, BoolOp, Not) that Phase 41 parsed but left as stubs returning "control flow not yet implemented". The work divides cleanly into two major areas: (1) evaluating the already-parsed control flow constructs, and (2) adding procedure definition/call infrastructure including `proc...end`, `local`, `option remember`, and `RETURN`.

The existing codebase provides strong foundations. The Pratt parser already handles `for/from/to/by/do/od`, `if/then/elif/else/fi`, and all comparison/boolean operators. The `AstNode` enum has `ForLoop`, `IfExpr`, `Compare`, `BoolOp`, and `Not` variants fully defined. The `Value` enum has 12 variants with `Clone + Debug` (no `PartialEq`). The `Environment` struct uses a flat `HashMap<String, Value>` for variables. The `dispatch()` function in eval.rs handles all 89+ built-in functions. The `FuncCall` AST node currently eagerly evaluates all arguments before calling `dispatch()`.

**Primary recommendation:** Add a `Value::Procedure` variant storing parameter names, local variable names, body AST, memoization flag, and a memo table. Parse `proc(...)...end` as a new AST node. Implement local scoping via save/restore on the existing `Environment.variables` HashMap (no need for a scope chain). Route `FuncCall` evaluation to check for user-defined procedures before falling through to `dispatch()`.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SCRIPT-02 | User can define named procedures: `f := proc(args) local x,k; stmts; end` | New Token variants (Proc, Local, Option), new AstNode::ProcDef, new Value::Procedure, parser proc rule, eval assignment stores procedure |
| SCRIPT-03 | User can use `option remember` in procedures for memoization | ProcDef stores `remember: bool`, Value::Procedure includes `memo: HashMap<Vec<ValueKey>, Value>` wrapped in RefCell, dispatch checks memo before executing body |
| SCRIPT-04 | User can use `RETURN(value)` to exit a procedure early | Treat RETURN as special function call in eval, use Rust error-as-control-flow pattern (EvalError::Return or dedicated ControlFlow enum) to unwind procedure body |
| SCRIPT-05 | User can use `local x, y` declarations for procedure-scoped variables | Parser reads `local` clause after proc parameters, eval saves/restores affected variable bindings around procedure body execution |
</phase_requirements>

## Standard Stack

### Core (no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std HashMap | - | Variable scoping (save/restore) | Already used in Environment |
| Rust std RefCell | - | Interior mutability for memo tables in cloned Value::Procedure | Required because Value is Clone but memo needs mutation |
| Rust std collections | - | BTreeMap or HashMap for memo keys | Already imported |

### No New Crates Required
This phase is purely internal to qsym-cli. No new crate dependencies are needed. The existing `qsym-core` types (QInt, QRat, FormalPowerSeries) and the CLI infrastructure (token, lexer, parser, eval, environment) provide everything needed.

## Architecture Patterns

### Recommended Change Locations

```
crates/qsym-cli/src/
  token.rs      -- Add Proc, Local, Option tokens (3 new variants)
  lexer.rs      -- Map "proc" -> Token::Proc, "local" -> Token::Local, "option" -> Token::Option
  ast.rs        -- Add AstNode::ProcDef variant
  parser.rs     -- Parse proc(args) local ...; option ...; body; end
  eval.rs       -- Value::Procedure variant, eval_compare/eval_bool/eval_for/eval_if,
                   procedure call dispatch, RETURN handling, local scoping
  environment.rs-- (minimal changes, scoping done in eval.rs)
  format.rs     -- Display for Value::Procedure ("proc(...) ... end proc")
  repl.rs       -- Multiline: track proc/end depth for incomplete detection
```

### Pattern 1: Control Flow Evaluation (already-parsed AST nodes)

**What:** Implement `eval_compare()`, `eval_bool_op()`, `eval_not()`, `eval_for_loop()`, `eval_if_expr()` in eval.rs to replace the current stub that returns `EvalError::Other("control flow not yet implemented")`.

**When to use:** The match arm at line 1031-1034 of eval.rs currently catches all five AST node types.

**Design:**
```rust
// In eval_expr(), replace the catch-all stub:
AstNode::Compare { op, lhs, rhs } => {
    let left = eval_expr(lhs, env)?;
    let right = eval_expr(rhs, env)?;
    eval_compare(*op, left, right)
}

AstNode::Not(inner) => {
    let val = eval_expr(inner, env)?;
    match val {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(EvalError::TypeError { ... })
    }
}

AstNode::BoolOp { op, lhs, rhs } => {
    // Short-circuit evaluation!
    let left = eval_expr(lhs, env)?;
    match (op, &left) {
        (BoolBinOp::And, Value::Bool(false)) => Ok(Value::Bool(false)),
        (BoolBinOp::Or, Value::Bool(true)) => Ok(Value::Bool(true)),
        _ => {
            let right = eval_expr(rhs, env)?;
            eval_bool_op(*op, left, right)
        }
    }
}

AstNode::ForLoop { var, from, to, by, body } => {
    eval_for_loop(var, from, to, by, body, env)
}

AstNode::IfExpr { condition, then_body, elif_branches, else_body } => {
    eval_if_expr(condition, then_body, elif_branches, else_body, env)
}
```

### Pattern 2: Comparison Evaluation

**What:** Compare two Values, producing `Value::Bool`. Maple comparisons work on integers and rationals.

**Design:**
```rust
fn eval_compare(op: CompOp, left: Value, right: Value) -> Result<Value, EvalError> {
    // Promote both to comparable types:
    // Integer vs Integer -> direct comparison
    // Integer vs Rational or Rational vs Integer -> promote to Rational
    // Symbol = Symbol -> name equality (for Maple compatibility)
    // Series: not comparable (error)
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => {
            let result = match op {
                CompOp::Eq => a == b,
                CompOp::NotEq => a != b,
                CompOp::Less => a < b,
                CompOp::Greater => a > b,
                CompOp::LessEq => a <= b,
                CompOp::GreaterEq => a >= b,
            };
            Ok(Value::Bool(result))
        }
        // ... Rational promotion, etc.
    }
}
```

### Pattern 3: For-Loop Evaluation

**What:** Evaluate a for-loop by iterating from start to end, optionally with step.

**Design:**
```rust
fn eval_for_loop(
    var: &str, from: &AstNode, to: &AstNode, by: &Option<Box<AstNode>>,
    body: &[Stmt], env: &mut Environment,
) -> Result<Value, EvalError> {
    let start = eval_to_i64(eval_expr(from, env)?)?;
    let end = eval_to_i64(eval_expr(to, env)?)?;
    let step = match by {
        Some(b) => eval_to_i64(eval_expr(b, env)?)?,
        None => 1,
    };
    // Save old value of loop variable
    let old_val = env.variables.remove(var);
    let mut last = Value::None;
    let mut i = start;
    while (step > 0 && i <= end) || (step < 0 && i >= end) {
        env.set_var(var, Value::Integer(QInt::from(i)));
        last = eval_stmt_sequence(body, env)?;
        i += step;
    }
    // Restore loop variable
    match old_val {
        Some(v) => env.set_var(var, v),
        None => { env.variables.remove(var); }
    }
    Ok(last)
}
```

**Key detail:** The loop variable must be scoped -- save its previous value before the loop and restore it after. Maple scopes the loop variable to the loop.

### Pattern 4: If Expression Evaluation

**What:** Evaluate if/elif/else chains, executing only the first matching branch.

**Design:**
```rust
fn eval_if_expr(
    condition: &AstNode, then_body: &[Stmt],
    elif_branches: &[(AstNode, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>, env: &mut Environment,
) -> Result<Value, EvalError> {
    let cond = eval_expr(condition, env)?;
    if is_truthy(&cond)? {
        return eval_stmt_sequence(then_body, env);
    }
    for (elif_cond, elif_body) in elif_branches {
        let cond = eval_expr(elif_cond, env)?;
        if is_truthy(&cond)? {
            return eval_stmt_sequence(elif_body, env);
        }
    }
    if let Some(else_stmts) = else_body {
        return eval_stmt_sequence(else_stmts, env);
    }
    Ok(Value::None)
}
```

### Pattern 5: Statement Sequence Evaluation

**What:** Helper to evaluate a `Vec<Stmt>`, returning the value of the last statement.

**Design:**
```rust
fn eval_stmt_sequence(stmts: &[Stmt], env: &mut Environment) -> Result<Value, EvalError> {
    let mut last = Value::None;
    for stmt in stmts {
        last = eval_expr(&stmt.node, env)?;
    }
    Ok(last)
}
```

**Note:** In procedure bodies, statement terminators (`;` vs `:`) do NOT affect which value is returned -- only the last evaluated expression matters. Terminators only affect printing in the REPL top level.

### Pattern 6: Procedure Definition (AST)

**What:** New AST node for procedure definitions parsed from `proc(args) local ...; option ...; body; end`.

**Design:**
```rust
// In ast.rs:
AstNode::ProcDef {
    params: Vec<String>,        // parameter names
    locals: Vec<String>,        // local variable names
    options: Vec<String>,       // e.g., ["remember"]
    body: Vec<Stmt>,            // procedure body statements
}
```

### Pattern 7: Procedure Value (Runtime)

**What:** New Value variant stored when `f := proc(...) ... end` is evaluated.

**Design:**
```rust
// In eval.rs:
use std::cell::RefCell;
use std::rc::Rc;

/// A user-defined procedure.
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: String,                          // assigned name (set on first :=)
    pub params: Vec<String>,                   // formal parameter names
    pub locals: Vec<String>,                   // local variable names
    pub body: Vec<Stmt>,                       // body statements (AST)
    pub remember: bool,                        // option remember?
    pub memo: Rc<RefCell<HashMap<Vec<String>, Value>>>,  // memo table
}

// Add to Value enum:
Value::Procedure(Procedure),
```

**Why Rc<RefCell<...>> for memo:** Value is Clone. When a procedure is stored in a variable, cloned, and called, all clones must share the same memo table. `Rc<RefCell<...>>` provides shared mutable state. The memo key is a serialized representation of argument values (e.g., via Debug string or a dedicated hash key).

### Pattern 8: Procedure Call Dispatch

**What:** When evaluating `FuncCall { name, args }`, check if `name` refers to a user-defined procedure before falling through to built-in dispatch.

**Design:**
```rust
// In eval_expr, modify the FuncCall arm:
AstNode::FuncCall { name, args } => {
    // Check for user-defined procedure FIRST
    if let Some(Value::Procedure(proc)) = env.get_var(name).cloned() {
        let mut evaluated = Vec::with_capacity(args.len());
        for arg in args {
            evaluated.push(eval_expr(arg, env)?);
        }
        return call_procedure(&proc, &evaluated, env);
    }
    // Fall through to built-in dispatch
    let mut evaluated = Vec::with_capacity(args.len());
    for arg in args {
        evaluated.push(eval_expr(arg, env)?);
    }
    dispatch(name, &evaluated, env)
}
```

**Critical ordering:** User procedures shadow built-in functions. This matches Maple behavior where you can redefine `numbpart` if you want.

### Pattern 9: Procedure Execution with Local Scoping

**What:** Execute a procedure body with parameter bindings and local variables, then restore the environment.

**Design:**
```rust
fn call_procedure(
    proc: &Procedure, args: &[Value], env: &mut Environment,
) -> Result<Value, EvalError> {
    // 1. Check argument count
    if args.len() != proc.params.len() {
        return Err(EvalError::WrongArgCount { ... });
    }

    // 2. Check memo table
    if proc.remember {
        let key = make_memo_key(args);
        if let Some(cached) = proc.memo.borrow().get(&key) {
            return Ok(cached.clone());
        }
    }

    // 3. Save variables that will be shadowed (params + locals)
    let mut saved: Vec<(String, Option<Value>)> = Vec::new();
    for name in proc.params.iter().chain(proc.locals.iter()) {
        saved.push((name.clone(), env.variables.remove(name)));
    }

    // 4. Bind parameters
    for (param, arg) in proc.params.iter().zip(args.iter()) {
        env.set_var(param, arg.clone());
    }
    // Locals start as unset (will become Symbol on access, or assigned in body)

    // 5. Execute body, catching RETURN
    let result = match eval_stmt_sequence_for_proc(&proc.body, env) {
        Ok(val) => Ok(val),
        Err(EvalError::EarlyReturn(val)) => Ok(val),
        Err(e) => Err(e),
    };

    // 6. Restore saved variables
    for (name, old_val) in saved {
        match old_val {
            Some(v) => env.set_var(&name, v),
            None => { env.variables.remove(&name); }
        }
    }

    // 7. Store in memo table if applicable
    if proc.remember {
        if let Ok(ref val) = result {
            let key = make_memo_key(args);
            proc.memo.borrow_mut().insert(key, val.clone());
        }
    }

    result
}
```

### Pattern 10: RETURN as Control Flow

**What:** `RETURN(expr)` exits a procedure early. Implemented as a special EvalError variant that unwinds the statement sequence.

**Design:**
```rust
// Add to EvalError:
EvalError::EarlyReturn(Value),  // Not a real error, control flow

// In eval_expr FuncCall arm (or in dispatch), before checking built-ins:
// RETURN is a special function
if name == "RETURN" {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount { ... });
    }
    let val = eval_expr(&args[0], env)?;
    return Err(EvalError::EarlyReturn(val));
}
```

**Why error-as-control-flow:** This is the standard pattern in tree-walking interpreters. RETURN needs to unwind through nested for-loops and if-expressions inside a procedure. The procedure call boundary catches EarlyReturn and converts it to Ok(value). Outside a procedure, RETURN produces an error message "RETURN used outside of procedure".

**Important:** RETURN must be handled BEFORE evaluating args in the FuncCall arm, because the argument should be evaluated but the result must propagate up. Actually, RETURN can be handled after evaluation since we need the argument value. The key is that RETURN evaluates its argument, then returns EarlyReturn.

### Pattern 11: Parsing `proc(...) ... end`

**What:** Parse procedure definitions in the Pratt parser.

**Grammar:**
```
proc_def := 'proc' '(' param_list ')' [local_decl] [option_decl] stmt_sequence 'end' ['proc']
param_list := ident (',' ident)*   // possibly empty
local_decl := 'local' ident (',' ident)* ';'
option_decl := 'option' ident (',' ident)* ';'
```

**Design:** Add `Token::Proc` as a keyword. When the parser sees `Token::Proc` as a prefix, parse the procedure structure. The tricky part is that `end` is already a token (`Token::End`). The parser should consume `end` and optionally `proc` after it (i.e., both `end` and `end proc` terminate the procedure).

```rust
// In expr_bp prefix handling:
Token::Proc => {
    self.advance(); // consume 'proc'
    self.expect(&Token::LParen, "'(' after 'proc'")?;
    let params = self.parse_ident_list()?;
    self.expect(&Token::RParen, "')' to close proc parameters")?;

    // Optional: local declarations
    let locals = if *self.peek() == Token::Local {
        self.advance();
        let names = self.parse_ident_list()?;
        self.expect(&Token::Semi, "';' after local declarations")?;
        names
    } else {
        Vec::new()
    };

    // Optional: option declarations
    let options = if *self.peek() == Token::Option {
        self.advance();
        let names = self.parse_ident_list()?;
        self.expect(&Token::Semi, "';' after option declarations")?;
        names
    } else {
        Vec::new()
    };

    // Body statements until 'end'
    let body = self.parse_stmt_sequence(&[Token::End])?;
    self.expect(&Token::End, "'end' to close proc")?;

    // Optional 'proc' after 'end' (Maple allows "end proc")
    if *self.peek() == Token::Proc {
        self.advance();
    }

    AstNode::ProcDef { params, locals, options, body }
}
```

### Anti-Patterns to Avoid

- **Scope chain (linked list of environments):** Overkill for this use case. Maple procedures do not capture lexical closures. Save/restore on the flat HashMap is simpler, faster, and matches Maple semantics.
- **Making RETURN a keyword/token:** RETURN is a function call in Maple syntax (`RETURN(x)`), not a statement keyword. It should be parsed as a regular `FuncCall` node and handled specially during evaluation.
- **Eager argument evaluation for RETURN:** `RETURN(expr)` must evaluate `expr` first, then unwind. The current FuncCall eval pattern (evaluate all args, then dispatch) works correctly -- the special handling happens in dispatch, which returns `EarlyReturn` after the arg is evaluated.
- **Storing procedure body as closures:** Rust closures cannot be easily cloned/stored. Store the AST (`Vec<Stmt>`) and re-evaluate it each call. This is the standard tree-walking interpreter approach.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Scope management | Linked scope chain, frame stack | Save/restore on HashMap | Simpler, no closures needed, matches Maple's flat scoping |
| Memo table key | Custom hash for Value | Debug-string key or dedicated ValueKey enum | Value doesn't implement Hash; Debug string is simplest starting point |
| Early return | Exception/panic-based control flow | EvalError::EarlyReturn variant | Clean Rust pattern, caught at procedure boundary |
| Procedure storage | Closure/function pointer | AST re-evaluation (Procedure struct with Vec<Stmt>) | Value must be Clone+Debug; AST is data, not code |

**Key insight:** This is a tree-walking interpreter, not a compiled language. Store AST, re-evaluate on each call. The memo table handles performance. Simplicity beats cleverness.

## Common Pitfalls

### Pitfall 1: Variable Leaking from Procedures
**What goes wrong:** After calling `f(5)`, the local variable `k` is visible in the global scope.
**Why it happens:** Forgetting to save/restore variables, or only saving params but not locals.
**How to avoid:** Save ALL names that the procedure will write to (params + locals) before execution, restore ALL of them after. Even on error paths.
**Warning signs:** Test `f := proc(n) local k; k := n; end; f(5); k;` -- k should be a symbol, not 5.

### Pitfall 2: RETURN Not Unwinding Through Nested Control Flow
**What goes wrong:** `RETURN(x)` inside a for-loop inside a procedure doesn't actually return from the procedure.
**Why it happens:** The for-loop evaluator catches the EarlyReturn error instead of propagating it.
**How to avoid:** Only catch `EarlyReturn` at the procedure call boundary (`call_procedure()`). For-loops and if-expressions must propagate EarlyReturn upward.
**Warning signs:** Test `f := proc(n) for k from 1 to 10 do if k = n then RETURN(k) fi od end; f(3)` should return 3.

### Pitfall 3: Memo Table Not Shared Across Clones
**What goes wrong:** Each call to a memoized procedure creates a fresh memo table (because Value::Clone clones the HashMap).
**Why it happens:** Using `HashMap` directly in the Procedure struct instead of `Rc<RefCell<HashMap>>`.
**How to avoid:** Wrap memo in `Rc<RefCell<...>>` so all clones share the same table.
**Warning signs:** Memoized procedure shows no speedup on repeated calls.

### Pitfall 4: `option` Conflicts with Token::Option
**What goes wrong:** The word "option" may be used as a variable name in existing scripts.
**Why it happens:** Adding "option" to the keyword list in the lexer.
**How to avoid:** "option" is indeed a Maple keyword, so making it a keyword is correct. However, be aware that `Token::Option` could conflict with Rust's `Option` type -- use a distinct name like `Token::OptionKw`.
**Warning signs:** Compilation error due to name collision.

### Pitfall 5: RETURN Outside Procedure
**What goes wrong:** User types `RETURN(5)` at top level and gets an opaque error.
**Why it happens:** EarlyReturn propagates all the way up to the REPL loop and isn't caught.
**How to avoid:** The REPL/script runner should catch `EarlyReturn` at the top level and convert it to a clear error: "RETURN used outside of a procedure".
**Warning signs:** Test typing `RETURN(5)` at the REPL.

### Pitfall 6: Colon Terminator Suppression in Procedure Body
**What goes wrong:** Procedure `f := proc() 1; 2: 3; end` -- what is returned?
**Why it happens:** Confusion about what `;` vs `:` means inside procedure bodies.
**How to avoid:** Inside a procedure body, terminators do NOT affect the return value. The return value is always the result of the last statement executed (regardless of `;` or `:`). Terminators only affect printing at REPL top-level.
**Warning signs:** Test that `f := proc() 1: end; f()` returns 1 (the colon doesn't suppress the return).

### Pitfall 7: For-Loop Variable Not Restored on Error
**What goes wrong:** If the loop body errors, the loop variable retains its last value.
**Why it happens:** Save/restore only happens on the success path.
**How to avoid:** Use a RAII-like pattern or ensure restore happens in all code paths (success, error, and early return).
**Warning signs:** After a failed for-loop, the loop variable has an unexpected value.

### Pitfall 8: Multiline REPL Detection for proc/end
**What goes wrong:** Typing `f := proc(n)` and pressing Enter doesn't give a continuation prompt.
**Why it happens:** The REPL validator (`is_incomplete()`) doesn't track proc/end depth.
**How to avoid:** Add proc/end tracking alongside for/od and if/fi in `ReplHelper::check_keyword()` and `is_incomplete()`.
**Warning signs:** Multiline procedure definitions require `\` continuation or single-line entry.

## Code Examples

### Example 1: Garvan-Style Procedure (from qseries package)
```maple
UE := proc(q, k, p, trunk)
  local x, m, n:
  x := 0:
  for m from 1 to trunk do
    for n from 1 to trunk/m do
      x := x + L(m, p) * n^(k-1) * q^(m*n):
    od:
  od:
end:
```
This shows: assignment to proc, 3 local variables, nested for-loops, colon terminators (suppress output). Source: Garvan qmaple tutorial.

### Example 2: Procedure with RETURN and option remember
```maple
fib := proc(n)
  option remember;
  if n <= 1 then RETURN(n) fi;
  fib(n-1) + fib(n-2);
end;
```
This shows: recursive proc, memoization, early return, if-conditional.

### Example 3: Expected q-Kangaroo Test Cases
```
# Define a simple procedure
f := proc(n) local k; k := n*n; k; end;
f(5);   # should return 25

# Verify local scoping
k;      # should return symbol 'k', not 25

# RETURN from nested control flow
g := proc(n) for k from 1 to 100 do if k = n then RETURN(k*k) fi od end;
g(7);   # should return 49

# option remember
fib := proc(n) option remember; if n <= 1 then RETURN(n) fi; fib(n-1) + fib(n-2); end;
fib(10);  # should return 55

# For loop evaluation
for n from 1 to 5 do n^2 od;  # last value is 25

# If/else evaluation
if 3 > 2 then 1 else 0 fi;  # should return 1
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Maple `end` only | Maple allows `end` or `end proc` | Maple 6+ | Parser should accept both forms |
| Maple `RETURN(x)` uppercase | Modern Maple also allows `return x` (lowercase, no parens) | Recent Maple | For Garvan compat, `RETURN(x)` is sufficient |
| Full closure support | q-Kangaroo: no lexical closures needed | Design decision | Simplifies implementation enormously |

**Deprecated/outdated:**
- Old Maple used `RETURN` (uppercase, function syntax). Modern Maple added lowercase `return` as a statement. For Garvan package compatibility, supporting `RETURN(expr)` as a function call is correct and sufficient.

## Open Questions

1. **Memo key representation**
   - What we know: Value doesn't implement Hash or Eq. We need a way to key the memo table.
   - What's unclear: Best key strategy -- Debug string? Dedicated ValueKey enum? Just integer args?
   - Recommendation: Start with Debug-string keys (`format!("{:?}", args)`). For q-series work, memoized procedures will typically have small integer arguments (like `fib(n)`, `UE(q,k,p,trunk)`), so Debug-string keys are adequate and simple. Can optimize later if profiling shows it matters.

2. **Nested procedure definitions**
   - What we know: Maple supports nested `proc` inside `proc`.
   - What's unclear: Whether Garvan's code uses this pattern.
   - Recommendation: Support it naturally -- since procedures are just AST values, a proc body can contain another ProcDef assignment. No special handling needed; the save/restore scoping already handles it.

3. **Procedure display format**
   - What we know: Maple displays procedures with their source code.
   - What's unclear: Exact display format desired.
   - Recommendation: Display as `proc(params) ... end proc` with a summary. For `format_value`, show `proc(n) ... end proc`. Full body display can be added later.

4. **Should local variables shadow built-in function names?**
   - What we know: Maple allows `local numbpart;` to shadow built-ins.
   - What's unclear: Whether this is needed for q-Kangaroo.
   - Recommendation: Yes, by design. Local variables are set in the environment HashMap, and procedure call dispatch checks the HashMap before `dispatch()`. This gives natural shadowing without extra work.

## Sources

### Primary (HIGH confidence)
- **Codebase inspection:** ast.rs, eval.rs (6729 lines), parser.rs, lexer.rs, token.rs, environment.rs, format.rs, repl.rs, script.rs -- all read and analyzed directly
- **ROADMAP.md** -- Phase 42 success criteria (5 items)
- **REQUIREMENTS.md** -- SCRIPT-02 through SCRIPT-05 definitions

### Secondary (MEDIUM confidence)
- [Maple procedure documentation](https://www.maplesoft.com/support/help/maple/view.aspx?path=procedure) -- Official Maple proc syntax
- [Maple Programming Guide Ch. 6](https://www.maplesoft.com/support/help/Maple/view.aspx?path=ProgrammingGuide/Chapter06) -- Procedure details
- [Garvan qmaple tutorial](https://qseries.org/fgarvan/papers/qmaple.pdf) -- Real-world procedure examples from the package this project reimplements
- [Stony Brook Maple procedures](https://www.math.stonybrook.edu/~scott/Book331/Defining_functions_with.html) -- local/global variable syntax
- [Ryerson Maple procedures](https://math.ryerson.ca/~danziger/professor/MTH207/Labs/less16.htm) -- RETURN and option remember examples

### Tertiary (LOW confidence)
- None needed; the implementation approach is straightforward tree-walking interpreter pattern, well-established in PL implementation literature.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, pure Rust internal work
- Architecture: HIGH - Patterns are standard tree-walking interpreter techniques, codebase is well-understood from direct inspection
- Pitfalls: HIGH - All identified from direct codebase analysis and Maple semantics
- Control flow eval: HIGH - AST nodes already exist, just need match arms
- Procedure parsing: HIGH - Follows same pattern as ForLoop/IfExpr parsing
- Memoization: MEDIUM - Memo key strategy (Debug string) is pragmatic but not elegant; may need refinement
- RETURN semantics: HIGH - Error-as-control-flow is the standard approach

**Research date:** 2026-02-20
**Valid until:** 2026-03-20 (stable domain, no external dependencies changing)
