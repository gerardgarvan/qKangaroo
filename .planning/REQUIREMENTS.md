# Requirements: q-Kangaroo

**Defined:** 2026-02-18
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

## v1.6 Requirements

Requirements for v1.6 CLI Hardening & Manual. Each maps to roadmap phases.

### Build Infrastructure

- [ ] **BUILD-01**: Binary has zero DLL dependencies (static GMP/MPFR/MPC linking)
- [ ] **BUILD-02**: CI workflow builds from bundled GMP source (no pre-installed system libs)
- [ ] **BUILD-03**: Release archive contains only the executable (no DLL files)

### CLI Flags

- [ ] **CLI-01**: User can run `q-kangaroo --help` / `-h` to see usage summary and exit
- [ ] **CLI-02**: User can run `q-kangaroo -q` / `--quiet` to suppress banner in interactive mode
- [ ] **CLI-03**: User can run `q-kangaroo -c "expr"` to evaluate an expression and exit
- [ ] **CLI-04**: User can run `q-kangaroo -v` / `--verbose` to see per-statement timing
- [ ] **CLI-05**: User can use `--` to separate options from positional filename arguments
- [ ] **CLI-06**: Unknown flags produce a clear error with `--help` suggestion and exit code 2

### Script Execution

- [ ] **EXEC-01**: User can run `q-kangaroo script.qk` to execute a script file and exit
- [ ] **EXEC-02**: Script files support `#` line comments
- [ ] **EXEC-03**: Script files support multi-line statements (unclosed parens span lines)
- [ ] **EXEC-04**: User can pipe input via stdin (`echo "expr" | q-kangaroo`)
- [ ] **EXEC-05**: Piped/script/`-c` modes automatically suppress banner and prompt
- [ ] **EXEC-06**: User can run `read("file.qk")` in the REPL to execute a file in the current session

### Exit Codes

- [ ] **EXIT-01**: Exit code 0 on success
- [ ] **EXIT-02**: Exit code 1 on evaluation error in batch mode
- [ ] **EXIT-03**: Exit code 2 on usage error (bad flags)
- [ ] **EXIT-04**: Exit code 65 on parse error in script input
- [ ] **EXIT-05**: Exit code 66 on file not found
- [ ] **EXIT-06**: Exit code 70 on caught panic (internal error)
- [ ] **EXIT-07**: Exit code 74 on I/O error

### Error Handling

- [ ] **ERR-01**: Script errors include `filename:line:col` context
- [ ] **ERR-02**: Common qsym-core panics are translated to human-readable messages
- [ ] **ERR-03**: File I/O errors display OS error message (not found, permission denied, etc.)
- [ ] **ERR-04**: Scripts fail-fast on first error (stop execution); REPL continues on error
- [ ] **ERR-05**: `read()` in REPL continues on error (matches interactive behavior)

### Documentation

- [ ] **DOC-01**: PDF reference manual covers all 81 functions with mathematical definitions
- [ ] **DOC-02**: Manual includes language reference, CLI usage, and session commands
- [ ] **DOC-03**: Manual includes worked examples and Maple migration guide
- [ ] **DOC-04**: PDF compiled from Typst source at release time
- [ ] **DOC-05**: PDF included in GitHub release archives alongside binary
- [ ] **DOC-06**: `--help` output mentions the PDF manual

## Future Requirements

Deferred to post-v1.6. Tracked but not in current roadmap.

### CLI Extensions

- **CLI-F01**: `-e` error-level flag (Maple-style -e0/-e1/-e2 control)
- **CLI-F02**: Config file support (`.q_kangaroorc` or `init.qk`)
- **CLI-F03**: Session recording / `save_session` command
- **CLI-F04**: Syntax highlighting in terminal (rustyline Highlighter trait)
- **CLI-F05**: `$include` / module/import system beyond `read()`

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| clap argument parser | Only 5-6 flags; hand-roll is simpler with zero dependencies |
| Embedded PDF in binary | Would bloat .exe by megabytes; ship as separate file |
| Interactive debugger / step mode | Overkill for math tool; users debug by evaluating subexpressions |
| First-run wizard/tutorial | Academic users don't expect this; Maple doesn't do it |
| macOS CI | Deferred to future milestone |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| (populated by roadmapper) | | |

**Coverage:**
- v1.6 requirements: 27 total
- Mapped to phases: 0
- Unmapped: 27

---
*Requirements defined: 2026-02-18*
*Last updated: 2026-02-18 after initial definition*
