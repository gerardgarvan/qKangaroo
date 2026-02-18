# Phase 26: REPL Shell & Session - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Interactive line-editing shell with history, tab completion, help system, and session configuration. The parser (Phase 24) and evaluator (Phase 25) are complete. LaTeX output and save-to-file commands are in Phase 27.

</domain>

<decisions>
## Implementation Decisions

### History & input behavior
- Prompt: `q> ` (short and clean)
- Welcome banner with ASCII art kangaroo, version number, and hint ("Type 'help' for commands, 'quit' to exit")
- History file stored alongside the executable (portable -- history travels with the binary)
- Claude's Discretion: multi-line input support (backslash continuation, auto-detect unclosed parens, or single-line only)

### Tab completion style
- Cycle through candidates with Tab (zsh-style), not bash-style list display
- Tab-completing a function name auto-inserts opening parenthesis: `aqp` → `aqprod(`
- Canonical function names only in completions -- no Maple aliases in Tab candidates (aliases still work when typed)
- Claude's Discretion: whether to also complete user-defined variable names

### Help system design
- Bare `help` shows grouped function list organized by category (Products, Partitions, Theta, Analysis, Relations, Hypergeometric, Mock Theta/Bailey, Identity Proving) with one-line descriptions
- `help` also includes a separate "Commands" section at bottom listing session commands (set, clear, quit, help, latex, save)
- Per-function help (`help aqprod`) shows signature + description + usage example with sample output
- No Maple alias mentions in help text -- keep it clean

### Session commands
- `clear` resets everything: variables, %, and precision back to default (20)
- `quit`, `exit`, and Ctrl-D all exit the REPL
- Session commands: set precision, clear, quit/exit, help -- no additional commands needed
- Claude's Discretion: whether `set precision N` affects only new computations or also retroactively recomputes stored variables

### Claude's Discretion
- Multi-line input approach
- Whether tab completes user-defined variables (in addition to functions)
- `set precision N` retroactive behavior

</decisions>

<specifics>
## Specific Ideas

- Welcome banner should have ASCII art of a kangaroo -- gives the tool personality and makes it memorable
- Prompt `q> ` is deliberately terse to maximize horizontal space for expressions
- History file next to executable means researchers can carry their REPL setup on a USB drive

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 26-repl-shell-session*
*Context gathered: 2026-02-17*
