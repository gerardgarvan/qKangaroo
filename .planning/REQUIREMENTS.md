# Requirements: q-Kangaroo

**Defined:** 2026-02-18
**Core Value:** Every function in Garvan's Maple packages works correctly in q-Kangaroo, producing matching output -- so researchers can switch without losing any capability.

## v1.6 Requirements

Requirements for v1.6 CLI Hardening & Manual. Each maps to roadmap phases.

### Build Infrastructure

- [x] **BUILD-01**: Binary has zero DLL dependencies (static GMP/MPFR/MPC linking)
- [x] **BUILD-02**: CI workflow builds from bundled GMP source (no pre-installed system libs)
- [x] **BUILD-03**: Release archive contains only the executable (no DLL files)

### CLI Flags

- [x] **CLI-01**: User can run `q-kangaroo --help` / `-h` to see usage summary and exit
- [x] **CLI-02**: User can run `q-kangaroo -q` / `--quiet` to suppress banner in interactive mode
- [x] **CLI-03**: User can run `q-kangaroo -c "expr"` to evaluate an expression and exit
- [x] **CLI-04**: User can run `q-kangaroo -v` / `--verbose` to see per-statement timing
- [x] **CLI-05**: User can use `--` to separate options from positional filename arguments
- [x] **CLI-06**: Unknown flags produce a clear error with `--help` suggestion and exit code 2

### Script Execution

- [x] **EXEC-01**: User can run `q-kangaroo script.qk` to execute a script file and exit
- [x] **EXEC-02**: Script files support `#` line comments
- [x] **EXEC-03**: Script files support multi-line statements (unclosed parens span lines)
- [x] **EXEC-04**: User can pipe input via stdin (`echo "expr" | q-kangaroo`)
- [x] **EXEC-05**: Piped/script/`-c` modes automatically suppress banner and prompt
- [x] **EXEC-06**: User can run `read("file.qk")` in the REPL to execute a file in the current session

### Exit Codes

- [x] **EXIT-01**: Exit code 0 on success
- [x] **EXIT-02**: Exit code 1 on evaluation error in batch mode
- [x] **EXIT-03**: Exit code 2 on usage error (bad flags)
- [x] **EXIT-04**: Exit code 65 on parse error in script input
- [x] **EXIT-05**: Exit code 66 on file not found
- [x] **EXIT-06**: Exit code 70 on caught panic (internal error)
- [x] **EXIT-07**: Exit code 74 on I/O error

### Error Handling

- [x] **ERR-01**: Script errors include `filename:line:col` context
- [x] **ERR-02**: Common qsym-core panics are translated to human-readable messages
- [x] **ERR-03**: File I/O errors display OS error message (not found, permission denied, etc.)
- [x] **ERR-04**: Scripts fail-fast on first error (stop execution); REPL continues on error
- [x] **ERR-05**: `read()` in REPL continues on error (matches interactive behavior)

### Documentation

- [x] **DOC-01**: PDF reference manual covers all 81 functions with mathematical definitions
- [x] **DOC-02**: Manual includes language reference, CLI usage, and session commands
- [x] **DOC-03**: Manual includes worked examples and Maple migration guide
- [x] **DOC-04**: PDF compiled from Typst source at release time
- [x] **DOC-05**: PDF included in GitHub release archives alongside binary
- [x] **DOC-06**: `--help` output mentions the PDF manual

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
| BUILD-01 | Phase 29 | Complete |
| BUILD-02 | Phase 29 | Complete |
| BUILD-03 | Phase 29 | Complete |
| CLI-01 | Phase 30 | Complete |
| CLI-02 | Phase 30 | Complete |
| CLI-03 | Phase 30 | Complete |
| CLI-04 | Phase 30 | Complete |
| CLI-05 | Phase 30 | Complete |
| CLI-06 | Phase 30 | Complete |
| EXEC-01 | Phase 30 | Complete |
| EXEC-02 | Phase 30 | Complete |
| EXEC-03 | Phase 30 | Complete |
| EXEC-04 | Phase 30 | Complete |
| EXEC-05 | Phase 30 | Complete |
| EXEC-06 | Phase 30 | Complete |
| EXIT-01 | Phase 31 | Complete |
| EXIT-02 | Phase 31 | Complete |
| EXIT-03 | Phase 31 | Complete |
| EXIT-04 | Phase 31 | Complete |
| EXIT-05 | Phase 31 | Complete |
| EXIT-06 | Phase 31 | Complete |
| EXIT-07 | Phase 31 | Complete |
| ERR-01 | Phase 31 | Complete |
| ERR-02 | Phase 31 | Complete |
| ERR-03 | Phase 31 | Complete |
| ERR-04 | Phase 31 | Complete |
| ERR-05 | Phase 31 | Complete |
| DOC-01 | Phase 32 | Complete |
| DOC-02 | Phase 32 | Complete |
| DOC-03 | Phase 32 | Complete |
| DOC-04 | Phase 32 | Complete |
| DOC-05 | Phase 32 | Complete |
| DOC-06 | Phase 32 | Complete |

**Coverage:**
- v1.6 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0

---
*Requirements defined: 2026-02-18*
*Last updated: 2026-02-18 after roadmap creation*
