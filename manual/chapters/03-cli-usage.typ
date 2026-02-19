// 03-cli-usage.typ -- Command-Line Interface (3-4 pages)

= Command-Line Interface
#index[command-line interface]

q-Kangaroo supports four execution modes: interactive REPL, script execution,
expression evaluation, and piped input. This chapter documents all command-line
flags, execution modes, session commands, and exit codes.

== Synopsis

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("q-kangaroo [OPTIONS] [FILE]
q-kangaroo -c EXPRESSION
command | q-kangaroo", lang: none)
]

== Options
#index[flags]

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Flag*], [*Description*],
  ),
  [`-h`, `--help`],
  [Show usage information and exit.],

  [`-V`, `--version`],
  [Print version string and exit.],

  [`-c EXPRESSION`],
  [Evaluate the given expression, print the result, and exit. The expression
   argument is required.],

  [`-q`, `--quiet`],
  [Suppress the ASCII banner in interactive mode. Has no effect in other modes.],

  [`-v`, `--verbose`],
  [Show per-statement execution timing on stderr. Works in all modes.],

  [`--`],
  [End of options. The next positional argument is treated as a filename, even
   if it starts with `-`.],
)

== Execution Modes
#index[interactive mode]
#index[script mode]

q-Kangaroo automatically selects its execution mode based on how it is invoked:

=== Interactive REPL

When invoked with no file argument and stdin is a terminal, q-Kangaroo enters
the interactive Read-Eval-Print Loop. Features include:

- Line editing with Emacs keybindings (via rustyline)
- Persistent command history (stored next to the executable)
- Tab completion for function names and user variables
- Multi-line input (open parentheses continue to the next line)
- Session commands (see below)

#repl-block("$ q-kangaroo
                                      /)
                                    '  \\
       ... (ASCII kangaroo banner) ...

Type 'help' for commands
'quit' to exit

q> partition_gf(5)
1 + q + 2*q^2 + 3*q^3 + 5*q^4 + O(q^5)")

=== Script Execution

Pass a filename as a positional argument to execute a script:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("$ q-kangaroo script.qk", lang: none)
]

Scripts use the same expression language as the REPL. Lines starting with `#`
are comments. The script stops on the first error (fail-fast semantics).
Error messages include `filename:line:` context.

=== Expression Evaluation

The `-c` flag evaluates a single expression:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("$ q-kangaroo -c \"etaq(1, 1, 20)\"
q^(1/24) * (1 - q - q^2 + q^5 + q^7 + ... + O(q^20))", lang: none)
]

Multiple statements can be separated by `;` within the expression string.

=== Piped Input

When stdin is not a terminal, q-Kangaroo reads all input and evaluates it
as a script:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("$ echo \"1 + 1\" | q-kangaroo
2", lang: none)
]

== Session Commands
#index[session commands]

In interactive mode, the following commands are recognized before the
expression parser. They are case-insensitive.

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Command*], [*Description*],
  ),
  [`help`],
  [Display all 81 functions grouped by category, plus session commands.],

  [`help `_function_],
  [Show detailed help for a specific function: signature, description, and example.],

  [`set precision N`],
  [Set the default truncation order for series computation. The default is 20.
   The value must be a positive integer.],

  [`clear`],
  [Reset all user variables, the last result (`%`), and the truncation order
   back to 20.],

  [`quit` / `exit`],
  [Exit the REPL. Ctrl-D (EOF) also exits. Ctrl-C cancels the current line
   without exiting.],

  [`latex` _[var]_],
  [Display the LaTeX representation of the last result, or of a named variable.],

  [`save` _filename_],
  [Save the text representation of the last result to a file.],

  [`read` _filename_],
  [Load and execute a script file within the current session. Variables defined
   in the script become available at the REPL prompt. Alternatively, use the
   function form `read("filename.qk")` in expressions.],
)

== Exit Codes
#index[exit codes]

q-Kangaroo uses sysexits-compatible exit codes for scripting integration:

#table(
  columns: (auto, auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Code*], [*Name*], [*Meaning*],
  ),
  [`0`], [Success], [All statements executed successfully.],
  [`1`], [Eval error], [A runtime evaluation error occurred (e.g., undefined variable, type mismatch, division by zero).],
  [`2`], [Usage error], [Invalid command-line arguments (unknown flag, missing `-c` argument).],
  [`65`], [Parse error], [Syntax error in input (unmatched parenthesis, invalid token, etc.).],
  [`66`], [File not found], [Script file does not exist or is unreadable.],
  [`70`], [Panic], [An internal computation error was caught (e.g., overflow in the core library).],
  [`74`], [I/O error], [File I/O failure other than "not found" (permission denied, disk full, etc.).],
)

In interactive mode, errors are displayed and the REPL continues. In script
and expression modes, execution stops on the first error and the appropriate
exit code is returned.

== Error Messages

#index[error messages]

In script and expression modes, parse errors include source location context:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("script.qk:3:5: unexpected token ')'", lang: none)
]

Evaluation errors in scripts include the filename and line number:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("script.qk:7: undefined variable 'x'", lang: none)
]

In interactive mode, errors are printed without filename context and the
REPL continues accepting input. Panics from the core library are caught
and translated to user-friendly messages.
