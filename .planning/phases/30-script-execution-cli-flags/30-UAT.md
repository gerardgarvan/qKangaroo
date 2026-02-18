---
status: complete
phase: 30-script-execution-cli-flags
source: 30-01-SUMMARY.md, 30-02-SUMMARY.md, 30-03-SUMMARY.md
started: 2026-02-18T21:30:00Z
updated: 2026-02-18T21:50:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Help flag shows usage
expected: Running `q-kangaroo --help` prints a usage summary listing all flags (-h, -V, -q, -v, -c, --), file argument syntax, and examples. Exits 0.
result: issue
reported: "Typing --help at the q> REPL prompt gives 'Error: undefined variable help' instead of showing help. CLI flag works from command line."
severity: minor

### 2. Version flag
expected: Running `q-kangaroo --version` prints version string containing "q-kangaroo" and a version number. Exits 0.
result: pass

### 3. Expression mode (-c)
expected: Running `q-kangaroo -c "1 + 1"` prints `2` to stdout with no banner or prompt, then exits.
result: pass

### 4. Script file execution
expected: Create a file `test.qk` containing `x := 10: y := 20: x + y` and run `q-kangaroo test.qk`. Prints `30` with no banner, then exits.
result: pass

### 5. Script with # comments
expected: Create a file with `# comment line` followed by `1 + 2`. Running it prints `3` — comments are ignored.
result: pass

### 6. Multi-line expression in script
expected: A script containing a function call split across multiple lines (e.g., `aqprod(\n  1,1,\n  1,infinity,20\n)`) parses and evaluates correctly.
result: pass

### 7. Piped stdin
expected: Running `echo "partition_count(5)" | q-kangaroo` prints `7` with no banner or prompt.
result: pass

### 8. Quiet mode (-q)
expected: Running `q-kangaroo -q -c "1+1"` outputs `2` without error. The -q flag is accepted and suppresses the banner.
result: pass

### 9. Verbose mode (-v)
expected: Running `q-kangaroo -v -c "1 + 1"` prints `2` on stdout and timing info (like `[0.001s]`) on stderr.
result: pass

### 10. Unknown flag error
expected: Running `q-kangaroo --badopt` exits with code 2 and prints an error message containing "unknown option" and a suggestion to use `--help`.
result: pass

### 11. read() function via -c
expected: Create `helper.qk` with `x := 42:` then run `q-kangaroo -c 'read("helper.qk"): x'`. Prints `42` — the read function loaded the file into the environment.
result: pass

## Summary

total: 11
passed: 10
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Typing --help at the REPL prompt should show help or suggest using the help command"
  status: failed
  reason: "User reported: Typing --help at the q> REPL prompt gives 'Error: undefined variable help' instead of showing help"
  severity: minor
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
