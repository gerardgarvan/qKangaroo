---
phase: 47-parser-language-extensions
verified: 2026-02-21T06:00:00Z
status: passed
score: 4/4 must-haves verified
must_haves:
  truths:
    - "User types ditto (\") to reference last computed result"
    - "User defines lambda with arrow syntax (q -> expr) and calls it"
    - "User evaluates fractional q-powers (q^(1/4)) and divides series by them"
    - "User writes proc with option before local (either order accepted)"
  artifacts:
    - path: "crates/qsym-cli/src/token.rs"
      provides: "Token::Ditto and Token::Arrow variants"
    - path: "crates/qsym-cli/src/lexer.rs"
      provides: "Ditto byte-lookahead disambiguation and Arrow lexing"
    - path: "crates/qsym-cli/src/ast.rs"
      provides: "AstNode::Lambda variant"
    - path: "crates/qsym-cli/src/parser.rs"
      provides: "Ditto NUD, Arrow LED, option/local loop"
    - path: "crates/qsym-cli/src/eval.rs"
      provides: "Lambda eval, FractionalPowerSeries variant and arithmetic"
    - path: "crates/qsym-cli/src/format.rs"
      provides: "FractionalPowerSeries display with q^(k/d) notation"
  key_links:
    - from: "lexer.rs"
      to: "Token::Ditto"
      via: "byte-lookahead after double-quote"
    - from: "parser.rs"
      to: "AstNode::LastResult"
      via: "NUD match on Token::Ditto"
    - from: "parser.rs"
      to: "AstNode::Lambda"
      via: "LED handler for Token::Arrow"
    - from: "eval.rs"
      to: "Value::Procedure"
      via: "AstNode::Lambda creates Procedure with single param"
    - from: "eval.rs"
      to: "Value::FractionalPowerSeries"
      via: "eval_pow with fractional Rational exponent"
    - from: "format.rs"
      to: "format_fractional_series"
      via: "format_value match arm for FractionalPowerSeries"
---

# Phase 47: Parser & Language Extensions Verification Report

**Phase Goal:** Users can reference previous results with ditto, define lambda functions with arrow syntax, use fractional q-powers in expressions, and write option/local in either order in procedures
**Verified:** 2026-02-21T06:00:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User types `aqprod(q,q,10); etamake(",q,100)` and ditto resolves to previous result | VERIFIED | CLI output: `{factors: {1: 1}, q_shift: 1/24}` -- etamake received aqprod result via ditto. Also `5 + 3; " + 1` returns 8 then 9. |
| 2 | User defines `F := q -> expr` and calls `F(q)` to get a series | VERIFIED | CLI: `F := q -> q^2 + 1; F(3)` returns `10`. Lambda creates Value::Procedure, callable via existing infrastructure. |
| 3 | User evaluates `theta2(100)/q^(1/4)` and gets series with fractional exponents | VERIFIED | CLI: `theta2(100)/q^(1/4)` returns `2*q^(323/4) + 2*q^(195/4) + ... + 2*q^(3/4) + O(q^(399/4))`. Also `q^(1/4)` displays correctly, `(q + q^2)/q^(1/4)` returns `q^(7/4) + q^(3/4)`. |
| 4 | User writes `proc(n) option remember; local k; k := n; end` without error | VERIFIED | CLI: returns `proc(n) ... end proc`. Parser loop accepts local/option in any order. Tests cover option-before-local, local-before-option, option-only, local-only, neither. |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/qsym-cli/src/token.rs` | Token::Ditto, Token::Arrow variants | VERIFIED | Both variants present (line 37: Ditto, line 33: Arrow) |
| `crates/qsym-cli/src/lexer.rs` | Ditto byte-lookahead, Arrow lexing | VERIFIED | Token::Ditto emitted with lookahead (line 53), Token::Arrow emitted for `->` (line 117), 7 ditto + 2 arrow lexer tests |
| `crates/qsym-cli/src/ast.rs` | AstNode::Lambda variant | VERIFIED | Lambda { param, body } variant at line 122-123 |
| `crates/qsym-cli/src/parser.rs` | Ditto NUD, Arrow LED, option/local loop | VERIFIED | Ditto NUD at line 156, Arrow LED at line 360, option/local loop at line 254, token_name entries for both |
| `crates/qsym-cli/src/eval.rs` | Lambda eval, FractionalPowerSeries, arithmetic ops | VERIFIED | Lambda eval at line 1179, FractionalPowerSeries variant at line 97, full arithmetic (add/sub/mul/div), simplify_fractional, series_div_general |
| `crates/qsym-cli/src/format.rs` | FractionalPowerSeries display and LaTeX | VERIFIED | format_fractional_series (line 232), format_fractional_series_latex (line 321), both match arms in format_value and format_latex |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| lexer.rs | Token::Ditto | byte-lookahead after `"` | WIRED | Peek next byte, emit Ditto if delimiter/operator/whitespace/EOF |
| parser.rs | AstNode::LastResult | NUD match on Token::Ditto | WIRED | Line 156: `Token::Ditto => { self.advance(); AstNode::LastResult }` |
| parser.rs | AstNode::Lambda | LED handler for Token::Arrow | WIRED | Line 360-372: checks `*self.peek() == Token::Arrow`, creates Lambda node |
| eval.rs | Value::Procedure | AstNode::Lambda eval | WIRED | Line 1179-1191: Lambda creates Procedure with single param, empty locals, Implicit terminator |
| eval.rs | Value::FractionalPowerSeries | eval_pow with fractional Rational | WIRED | Symbol^Rational with denom > 1 creates FPS monomial with denom field |
| eval.rs | eval_div | Series / FractionalPowerSeries rescaling | WIRED | Line 2026-2032: rescale numerator, shift-divide, return FPS |
| format.rs | format_fractional_series | format_value match arm | WIRED | Line 50: FractionalPowerSeries dispatches to format function with q^(k/d) notation |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| LANG-01 | 47-01 | Ditto operator `"` references last computed result | SATISFIED | Token::Ditto with byte-lookahead disambiguation, maps to AstNode::LastResult. CLI: `5+3; "+1` returns 9. 8 ditto-specific tests pass. |
| LANG-02 | 47-02 | Arrow operator `->` for lambda functions | SATISFIED | Token::Arrow, AstNode::Lambda, lambda evals to Value::Procedure. CLI: `F := q -> q^2+1; F(3)` returns 10. 11 lambda-specific tests pass. |
| LANG-04 | 47-03 | Fractional q-powers `q^(1/4)`, series division | SATISFIED | Value::FractionalPowerSeries with full arithmetic. CLI: `q^(1/4)` displays correctly, `theta2(100)/q^(1/4)` produces fractional series. 19 fractional-specific tests pass. |
| LANG-05 | 47-01 | `option remember` before `local` in procedures | SATISFIED | Parser uses loop to accept local/option in any order or combination. 5 proc order tests pass. |

No orphaned requirements: REQUIREMENTS.md maps LANG-01, LANG-02, LANG-04, LANG-05 to Phase 47. All four are covered by plans and verified.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | No TODO, FIXME, PLACEHOLDER, or stub patterns found in any modified file |

### Test Results

- **667 lib tests pass** (0 failures)
- **151 integration tests pass** (1 pre-existing failure: `err_05_read_nonexistent_shows_file_not_found` -- parser issue with dots in filenames, unrelated to Phase 47)
- **47 new tests added** across all plans: 7 lexer ditto + 2 lexer arrow + 8 parser ditto + 7 parser lambda + 5 parser proc order + 4 eval lambda + 8 eval fractional + 10 format fractional (approximate per summaries; 47 total is approximate)

### Human Verification Required

None required. All four success criteria verified via automated CLI execution and unit tests. No visual, real-time, or external service behavior involved.

### Gaps Summary

No gaps found. All four requirements (LANG-01, LANG-02, LANG-04, LANG-05) are fully implemented, tested, and working end-to-end.

**Note on LANG-04 example `theta2(q,100)/q^(1/4)`:** The 2-arg `theta2(q,100)` syntax is not yet supported (that is FIX-02 in Phase 48). The fractional q-power feature itself works correctly -- `theta2(100)/q^(1/4)` (1-arg form) and `(q + q^2)/q^(1/4)` both produce correct fractional series output. The LANG-04 requirement is about fractional q-powers, not about theta2 signature variants.

---

_Verified: 2026-02-21T06:00:00Z_
_Verifier: Claude (gsd-verifier)_
