---
status: diagnosed
phase: 52-bug-fix-language-extensions
source: [52-01-SUMMARY.md, 52-02-SUMMARY.md]
started: 2026-02-22T03:35:00Z
updated: 2026-02-22T03:50:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Polynomial division does not hang
expected: `1/aqprod(q,q,5)` returns a series result without hanging. Should complete instantly and display a power series in q.
result: pass

### 2. For-loop with polynomial division completes
expected: `for n from 1 to 5 do q^n/aqprod(q,q,n) od` completes in bounded time and returns the last division result as a series.
result: pass

### 3. Unicode paste resilience
expected: Pasting Unicode math operators parses correctly. `q∧5` parses as `q^5`, `3 × 5` gives `15`, `10 − 3` gives `7`.
result: pass

### 4. print() displays intermediate values
expected: `for n from 1 to 5 do print(n) od` displays each value 1 through 5 on separate lines during execution, then returns 5 as the final result.
result: pass

### 5. Basic while-loop
expected: `i:=0: while i<10 do i:=i+1 od: i;` evaluates to `10`.
result: pass

### 6. While-loop doubling
expected: `x:=1: while x<100 do x:=x*2 od: x;` evaluates to `128`.
result: pass

### 7. While-loop safety limit
expected: `while true do 1 od` hits the iteration safety limit and displays an error mentioning "maximum iteration count" (1000000). Should NOT hang.
result: issue
reported: "while true do 1 od gives Error: expected boolean or integer in condition, got symbol"
severity: major

### 8. REPL multiline while detection
expected: Typing `while x < 10 do` and pressing Enter shows a continuation prompt (the line is incomplete). Then typing `x := x+1 od` completes the block.
result: pass

### 9. ?while help entry
expected: `?while` in the REPL displays help text showing `while condition do body od` syntax with examples.
result: issue
reported: "nothing happens it just goes to an empty line"
severity: major

## Summary

total: 9
passed: 7
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "while true do 1 od hits safety limit and errors with max iteration count message"
  status: failed
  reason: "User reported: while true do 1 od gives Error: expected boolean or integer in condition, got symbol"
  severity: major
  test: 7
  root_cause: "true is not a keyword — lexer tokenizes it as Token::Ident, parser creates AstNode::Variable, eval produces Value::Symbol('true'), and is_truthy() rejects symbols"
  artifacts:
    - path: "crates/qsym-cli/src/eval.rs"
      issue: "is_truthy() only handles Value::Bool and Value::Integer, not Symbol('true'/'false')"
    - path: "crates/qsym-cli/src/lexer.rs"
      issue: "No Token::True/Token::False keywords defined"
  missing:
    - "Make is_truthy handle Value::Symbol('true') as truthy and Value::Symbol('false') as falsy"
  debug_session: ""

- truth: "?while displays help text with syntax and examples"
  status: failed
  reason: "User reported: nothing happens it just goes to an empty line"
  severity: major
  test: 9
  root_cause: "parse_command() in commands.rs has no case for ? prefix — only handles 'help', 'quit', 'set', etc. The while help entry exists in help.rs but ? prefix dispatch is missing"
  artifacts:
    - path: "crates/qsym-cli/src/commands.rs"
      issue: "parse_command() does not check for ? prefix to route to help system"
  missing:
    - "Add ? prefix handling in parse_command() that extracts topic and returns Command::Help(Some(topic))"
  debug_session: ""
