---
phase: 31-error-hardening-exit-codes
type: uat
status: passed
started: 2026-02-18
completed: 2026-02-18
---

# Phase 31: Error Hardening & Exit Codes — UAT

## Test Plan

| # | Test | Requirement | Status |
|---|------|-------------|--------|
| 1 | File not found: exit code 66 with human-readable message | EXIT-05, ERR-03 | PASS |
| 2 | Script error shows filename:line:col context | ERR-01 | PASS |
| 3 | Panic translated to human-friendly message (no raw backtrace) | ERR-02 | PASS |
| 4 | Scripts fail-fast on first error | ERR-04 | PASS |
| 5 | Unknown flag shows error + --help suggestion, exit code 2 | EXIT-03 | PASS |
| 6 | Parse error in script: exit code 65 | EXIT-04 | PASS |
| 7 | read() in REPL shows clear error for missing file | ERR-05 | PASS |

## Test Results

### Test 1: File Not Found (PASS)
```
$ q-kangaroo nonexistent_file.qk
file not found: 'nonexistent_file.qk': The system cannot find the file specified. (os error 2)
EXIT: 66
```
Exit code 66, human-readable message with OS error detail.

### Test 2: Script Error with filename:line:col (PASS)
```
$ q-kangaroo /tmp/test_err.qk
C:/cygwin64/tmp/test_err.qk:2:1: parse error: expected ';', ':', or end of input, found identifier 'y'
  y := badvar
  ^
EXIT: 65
```
Shows filename:line:col and caret pointing to error location.

### Test 3: Panic Translation (PASS)
```
$ q-kangaroo -c "1/0"
Error: computation failed: division by zero
EXIT: 70

$ q-kangaroo -c "1/q"
Error: computation failed: cannot invert a series whose constant term is zero (the series starts at q^k with k > 0; try shifting or extracting the leading power first)
EXIT: 70
```
Both panics translated to human-friendly messages. No raw Rust backtrace. Exit code 70.

### Test 4: Fail-Fast (PASS)
```
$ q-kangaroo /tmp/test_failfast.qk   # contains: x := 1 + 2; undefined_a; undefined_b
3
C:/cygwin64/tmp/test_failfast.qk:1: Error: undefined variable 'undefined_a'
EXIT: 1
```
First statement evaluates, second errors, third (`undefined_b`) never reached. Fail-fast confirmed.

### Test 5: Unknown Flag (PASS)
```
$ q-kangaroo --bogus
q-kangaroo: unknown option '--bogus'
Try 'q-kangaroo --help' for more information.
EXIT: 2
```
Clear error with --help suggestion. Exit code 2.

### Test 6: Parse Error Exit 65 (PASS)
```
$ q-kangaroo /tmp/test_parse.qk   # contains: 1 + + 2
C:/cygwin64/tmp/test_parse.qk:1:5: parse error: expected expression, found '+'
  1 + + 2
      ^
EXIT: 65
```
Parse error with filename:line:col, caret, exit code 65.

### Test 7: read() Missing File (PASS)
```
$ q-kangaroo -c 'read("/tmp/no_such_file_xyz.qk")'
Error: file not found: '/tmp/no_such_file_xyz.qk': The system cannot find the file specified. (os error 2)
EXIT: 1
```
Clear "file not found" message (not "computation failed: panic"). Exit code 1 (eval error, not 66).
