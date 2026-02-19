// 04-expression-language.typ -- Expression Language (3-4 pages)

= Expression Language
#index[expression language]

q-Kangaroo uses a Maple-inspired expression language for all input, whether
typed at the interactive prompt, written in a script file, or passed via the
`-c` flag. This chapter describes the complete language syntax.

== Overview

The expression language supports integer and rational arithmetic, formal
power series in the indeterminate $q$, variable assignment, function calls,
lists, and two statement terminators that control output. There are no
control-flow statements (loops, conditionals) -- the language is designed
for evaluating mathematical expressions, not general-purpose programming.

== Literals

=== Integer Literals
#index[integer literals]

Integer literals are written in decimal notation. They have arbitrary
precision -- there is no upper limit on the number of digits:

#repl("999999999999999999999999999 + 1", "1000000000000000000000000000")

=== The `q` Indeterminate
#index[q indeterminate]

The symbol `q` is a reserved keyword representing the formal indeterminate
of power series. It cannot be used as a variable name:

#repl("q", "q")

#repl("q^2 + q + 1", "1 + q + q^2")

=== The `infinity` Keyword
#index[infinity]

The keyword `infinity` is used as an argument to functions that accept
either a finite bound or an infinite product:

#repl("aqprod(1, 1, 1, infinity, 10)", "1 - q - q^2 + q^5 + q^7 + O(q^10)")

=== String Literals

Double-quoted strings are used for filenames in the `read()` function and
the `save` command:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("q> read(\"script.qk\")", lang: none)
]

Supported escape sequences: `\\` (backslash), `\"` (double quote), `\n`
(newline), `\t` (tab).

== Variables and Assignment
#index[variables]
#index[assignment]

Variables are bound using the `:=` assignment operator:

#repl-block("q> f := partition_gf(20):
q> g := distinct_parts_gf(20):
q> f
1 + q + 2*q^2 + 3*q^3 + 5*q^4 + ... + O(q^20)")

Variable names consist of letters, digits, and underscores, and must start
with a letter. Names are case-sensitive: `f` and `F` are different variables.

=== Last Result Reference (`%`)
#index[last result]

The special symbol `%` refers to the value of the most recently printed
result:

#repl-block("q> partition_gf(10)
1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)
q> prodmake(%, 5)
{1: -1, 2: -1, 3: -1, 4: -1, 5: -1}")

== Arithmetic Operators
#index[operators]

#table(
  columns: (auto, auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Operator*], [*Syntax*], [*Description*],
  ),
  [`+`], [$a + b$], [Addition. Works on integers, rationals, and series.],
  [`-`], [$a - b$], [Subtraction. Also unary negation: $-a$.],
  [`*`], [$a * b$], [Multiplication. Series are multiplied with truncation.],
  [`/`], [$a \/ b$], [Division. Integer division produces a rational. Series
   division uses power series inversion.],
  [`^`], [$a hat b$], [Exponentiation. For series, the exponent must be an integer.
   For integers, both base and exponent must be non-negative.],
)

Operator precedence follows standard mathematical conventions:
`^` binds tightest, then `*` and `/`, then `+` and `-`. Parentheses
override precedence as usual.

#repl("2 + 3 * 4", "14")

#repl("(2 + 3) * 4", "20")

== Lists
#index[lists]

Lists are written with square brackets and comma-separated elements:

#repl-block("q> [1, 2, 3]
[1, 2, 3]")

Lists are used as arguments to several functions. For example, `findlincombo`
takes a list of candidate series, and `phi` / `psi` take lists of parameter
triples for the upper and lower parameters of hypergeometric series:

#repl-block("q> findlincombo(partition_gf(30), [distinct_parts_gf(30), odd_parts_gf(30)], 0)
[0, 1]")

== Function Calls

Functions are called with the standard `name(arg1, arg2, ...)` syntax.
q-Kangaroo provides 81 built-in functions organized into 8 groups:

- *Products* (7): `aqprod`, `qbin`, `etaq`, `jacprod`, `tripleprod`, `quinprod`, `winquist`
- *Partitions* (7): `partition_count`, `partition_gf`, `distinct_parts_gf`, `odd_parts_gf`, `bounded_parts_gf`, `rank_gf`, `crank_gf`
- *Theta Functions* (3): `theta2`, `theta3`, `theta4`
- *Series Analysis* (9): `sift`, `qdegree`, `lqdegree`, `qfactor`, `prodmake`, `etamake`, `jacprodmake`, `mprodmake`, `qetamake`
- *Relations* (12): `findlincombo`, `findhomcombo`, `findnonhomcombo`, and 9 others
- *Hypergeometric* (9): `phi`, `psi`, `try_summation`, `heine1`--`heine3`, `sears_transform`, `watson_transform`, `find_transformation_chain`
- *Mock Theta & Bailey* (27): 20 mock theta functions, 3 Appell-Lerch/universal, 4 Bailey chain
- *Identity Proving* (7): `prove_eta_id`, `search_identities`, `q_gosper`, `q_zeilberger`, `verify_wz`, `q_petkovsek`, `prove_nonterminating`

See Chapters 5--12 for complete documentation of every function.

== Statement Separators
#index[statement separators]

Multiple statements can appear on one line, separated by `;` or `:`:

- *Semicolon* (`;`): Evaluate and *print* the result.
- *Colon* (`:`): Evaluate and *suppress* the output.
- *End of line*: Implicitly prints the result (same as `;`).

#repl-block("q> a := 2; b := 3; a + b
2
3
5")

#repl-block("q> a := 2: b := 3: a + b
5")

In the second example, the assignments are suppressed by `:`, so only the
final expression `a + b` produces output.

== Comments
#index[comments]

In scripts, lines starting with `#` are comments:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("# This is a comment
f := etaq(1, 1, 20)  # inline comments also work
f", lang: none)
]

Comments extend from `#` to the end of the line.

== Value Types
#index[value types]

Every expression in q-Kangaroo evaluates to one of the following types:

#table(
  columns: (auto, 1fr),
  inset: 6pt,
  stroke: 0.5pt + luma(180),
  table.header(
    [*Type*], [*Description*],
  ),
  [Series],
  [A formal power series in $q$ with rational coefficients, truncated at a
   specified order. Produced by most q-series functions (`aqprod`, `etaq`,
   `partition_gf`, `theta3`, etc.).],

  [Integer],
  [An arbitrary-precision integer. Produced by `partition_count`, `qdegree`,
   `lqdegree`, and integer arithmetic.],

  [Rational],
  [An exact rational number $p\/q$. Produced by integer division and
   coefficient extraction.],

  [List],
  [An ordered collection of values. Produced by `[a, b, c]` syntax and
   functions like `findcong`.],

  [Dict],
  [A key-value mapping. Produced by `prodmake`, `etamake`, `qfactor`,
   `jacprodmake`, `mprodmake`, and `qetamake`.],

  [Pair],
  [A pair of two values. Produced by Heine transformations and Bailey
   lemma functions (which return `(prefactor, transformed_series)` or
   `(alpha, beta)` pairs).],

  [Bool],
  [A boolean value (`true` or `false`). Produced by `prove_eta_id` and
   `verify_wz`.],

  [String],
  [A text string. Used for filenames in `read()` and `save`.],

  [None],
  [The null value. Returned by `try_summation` when no closed form is found,
   and by `bailey_discover` when no proof is found.],

  [Infinity],
  [The `infinity` keyword. Used as a parameter to `aqprod` to request an
   infinite product.],
)

Arithmetic operations automatically promote types where sensible: adding an
integer to a series produces a series, dividing two integers produces a
rational, and so on.
