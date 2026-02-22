// 17-v5-changes.typ -- What's New in v5.0
#import "../template.typ": *

= What's New in v5.0
#index[v5.0]

q-Kangaroo v5.0 closes the remaining Maple language and function gaps.
This chapter documents all features introduced in Phases 52--55:
while loops, the `print()` function, list literals and indexing,
range-based iteration (`add`/`mul`/`seq`), sixteen new utility
functions, and a critical polynomial-division fix.

== Bug Fixes

=== Polynomial Division Fix
#index[polynomial division fix]
#index[POLYNOMIAL_ORDER]

Division by dense polynomials (e.g.~`1/aqprod(q,q,5)`) previously
could hang because the evaluator attempted infinite expansion of the
exact polynomial reciprocal. An internal `POLYNOMIAL_ORDER` sentinel
now prevents this: exact polynomials are automatically promoted to
truncated series before division, so the result is computed instantly.

#repl("series(1/aqprod(q,q,5), q, 20)",
  "164*q^19 + 141*q^18 + 119*q^17 + 101*q^16 + 84*q^15 + 70*q^14 + 57*q^13 + 47*q^12 + 37*q^11 + 30*q^10 + 23*q^9 + 18*q^8 + 13*q^7 + 10*q^6 + 7*q^5 + 5*q^4 + 3*q^3 + 2*q^2 + q + 1 + O(q^20)")

== Language Features

=== While Loops
#index[while loops]
#index-main[while]

Syntax: `while condition do body od`

The `while` loop repeatedly evaluates its body as long as the condition
is true. A safety limit of 1,000,000 iterations prevents accidental
infinite loops.

#repl-block("q> i := 0: while i < 10 do i := i + 1 od: i
10")

Unlike `for` loops, `while` does not introduce a new scope -- the loop
variable persists in the enclosing scope. Multi-line `while` blocks
are supported in the REPL (auto-detects incomplete input).

#repl-block("q> s := 1:
q> while s < 1000 do
q>   s := s * 2:
q> od:
q> s
1024")

=== The print() Function
#index-main[print]
#index[print function]

Syntax: `print(expr, ...)`

Displays one or more expressions, each on its own line. Useful for
showing intermediate values inside loops and procedures.

#repl("print(42)", "42\n42")

Unlike Maple's `print`, which returns `NULL`, q-Kangaroo's `print`
returns the last argument's value. This means `print(x)` both
displays _and_ returns `x`.

=== List Literals and Indexing
#index[list literals]
#index[list indexing]
#index-main[lists]

Syntax: `[a, b, c]` creates a list. Lists use 1-based indexing
following Maple convention.

#repl("L := [10, 20, 30]: L[2]", "20")

Nested lists are supported:

#repl("M := [[1,2],[3,4]]: M[2]", "[3, 4]")

Indexed assignment modifies a list element in place:

#repl-block("q> L := [10, 20, 30]:
q> L[2] := 99:
q> L
[10, 99, 30]")

Attempting `L[0]` produces an out-of-range error (indices start at 1).

=== Range Syntax (DotDot)
#index[range syntax]
#index[DotDot]

Syntax: `i=a..b` used in `add`, `mul`, and `seq`

The range expression `i=a..b` specifies an iteration variable and
bounds. It is only valid inside `add`, `mul`, or `seq` -- using it
elsewhere produces a clear error message.

#repl("add(i^2, i=1..5)", "55")

#repl("seq(2^k, k=0..4)", "[1, 2, 4, 8, 16]")

=== Unicode Paste Resilience
#index[Unicode normalization]

Unicode math characters (minus sign U+2212, various caret and dash
variants) are automatically normalized to their ASCII equivalents
before tokenization. This means researchers can paste expressions
from PDFs and web pages without encountering syntax errors.

== New Functions

=== List Operations

#func-entry(
  name: "nops",
  signature: "nops(expr)",
  description: [
    Return the number of operands. For a list, this is the number
    of elements. For a formal power series, this counts the number
    of nonzero terms (using the sparse storage directly).
    #index[nops]
  ],
  params: (
    ([expr], [List / Series / Any], [Expression to inspect]),
  ),
  examples: (
    ("nops([10, 20, 30])", "3"),
    ("nops(aqprod(q,q,infinity,20))", "7"),
  ),
  related: ("op", "map", "sort"),
)

#func-entry(
  name: "op",
  signature: "op(i, expr)",
  description: [
    Extract the $i$-th operand of an expression. For a list,
    returns the $i$-th element (1-indexed). For a formal power
    series, returns a `[exponent, coefficient]` pair for the
    $i$-th nonzero term.
    #index[op]
  ],
  params: (
    ([i], [Integer], [1-based operand index]),
    ([expr], [List / Series / Any], [Expression to decompose]),
  ),
  examples: (
    ("op(1, [10, 20, 30])", "10"),
    ("op(2, [10, 20, 30])", "20"),
  ),
  related: ("nops", "map"),
)

#func-entry(
  name: "map",
  signature: "map(f, list)",
  description: [
    Apply a function to each element of a list, returning a new
    list. The function `f` can be a named function, a procedure,
    or an arrow expression.
    #index[map]
  ],
  params: (
    ([f], [Function / Procedure], [Function to apply]),
    ([list], [List], [Input list]),
  ),
  examples: (
    ("map(x -> x^2, [1, 2, 3, 4])", "[1, 4, 9, 16]"),
  ),
  related: ("nops", "op", "sort", "seq"),
)

#func-entry(
  name: "sort",
  signature: "sort(list)",
  description: [
    Sort a list numerically (for numbers) or lexicographically
    (for strings/symbols). Returns a new sorted list.
    #index[sort]
  ],
  params: (
    ([list], [List], [List to sort]),
  ),
  examples: (
    ("sort([3, 1, 4, 1, 5])", "[1, 1, 3, 4, 5]"),
  ),
  related: ("nops", "op", "map"),
)

=== Series Coefficients

#func-entry(
  name: "coeff",
  signature: "coeff(f, q, n)",
  description: [
    Extract the coefficient of $q^n$ in the series $f$. Returns an
    `Integer` when the coefficient has denominator 1, and a
    `Rational` otherwise.
    #index-main[coeff]
  ],
  params: (
    ([f], [Series], [A formal power series]),
    ([q], [Variable], [The series variable]),
    ([n], [Integer], [Power to extract]),
  ),
  examples: (
    ("coeff(1 + 2*q + 3*q^2, q, 2)", "3"),
    ("coeff(aqprod(q,q,infinity,20), q, 5)", "1"),
  ),
  related: ("degree", "series"),
)

#func-entry(
  name: "degree",
  signature: "degree(f, q)",
  description: [
    Return the highest power of $q$ with a nonzero coefficient
    in the series $f$.
    #index-main[degree]
  ],
  params: (
    ([f], [Series / Polynomial], [A series or polynomial]),
    ([q], [Variable], [The series variable]),
  ),
  examples: (
    ("degree(1 + 2*q + 3*q^2, q)", "2"),
  ),
  related: ("coeff", "series"),
)

=== Rational Decomposition

#func-entry(
  name: "numer",
  signature: "numer(r)",
  description: [
    Return the numerator of a rational number.
    #index-main[numer]
  ],
  params: (
    ([r], [Rational / Integer], [A rational or integer value]),
  ),
  examples: (
    ("numer(3/7)", "3"),
  ),
  related: ("denom",),
)

#func-entry(
  name: "denom",
  signature: "denom(r)",
  description: [
    Return the denominator of a rational number.
    For integers, returns 1.
    #index-main[denom]
  ],
  params: (
    ([r], [Rational / Integer], [A rational or integer value]),
  ),
  examples: (
    ("denom(3/7)", "7"),
    ("denom(42)", "1"),
  ),
  related: ("numer",),
)

=== Modular Arithmetic

#func-entry(
  name: "modp",
  signature: "modp(a, p)",
  description: [
    Compute $a mod p$ with a non-negative result in the range
    $[0, p)$. Handles negative values correctly.
    #index-main[modp]
  ],
  params: (
    ([a], [Integer], [Dividend]),
    ([p], [Integer], [Modulus (positive)]),
  ),
  examples: (
    ("modp(7, 3)", "1"),
    ("modp(-7, 3)", "2"),
  ),
  related: ("mods",),
)

#func-entry(
  name: "mods",
  signature: "mods(a, p)",
  description: [
    Compute $a mod p$ with a symmetric (centered) result in the
    range $(-p\/2, p\/2]$. This is the balanced or centered
    modular reduction.
    #index-main[mods]
  ],
  params: (
    ([a], [Integer], [Dividend]),
    ([p], [Integer], [Modulus (positive)]),
  ),
  examples: (
    ("mods(5, 3)", "-1"),
    ("mods(4, 3)", "1"),
  ),
  related: ("modp",),
)

=== Type and Evaluation

#func-entry(
  name: "type",
  signature: "type(expr, typename)",
  description: [
    Check whether `expr` has the specified type. The `typename`
    argument can be a Symbol (e.g.~`integer`) or a String
    (e.g.~`"integer"`). Returns `true` or `false`. Unknown type
    names return `false` rather than raising an error.
    #index-main[type]
  ],
  params: (
    ([expr], [Any], [Expression to test]),
    ([typename], [Symbol / String], [Type name: `integer`, `rational`, `series`, `list`, `string`, etc.]),
  ),
  examples: (
    ("type(42, integer)", "true"),
    ("type(3/7, rational)", "true"),
  ),
  related: ("evalb",),
)

#func-entry(
  name: "evalb",
  signature: "evalb(expr)",
  description: [
    Evaluate a boolean expression. Forces evaluation of comparison
    operators that might otherwise remain symbolic.
    #index-main[evalb]
  ],
  params: (
    ([expr], [Boolean], [A boolean or comparison expression]),
  ),
  examples: (
    ("evalb(3 > 2)", "true"),
  ),
  related: ("type",),
)

=== String and Name Operations

#func-entry(
  name: "cat",
  signature: "cat(a, b, ...)",
  description: [
    Concatenate the string representations of all arguments into
    a single Symbol (not a String), matching Maple's `cat`
    behaviour.
    #index-main[cat]
  ],
  params: (
    ([a, b, ...], [Any], [Values to concatenate]),
  ),
  examples: (
    ("cat(hello, world)", "helloworld"),
    ("cat(x, 3)", "x3"),
  ),
  related: ("type",),
)

=== Iteration

#func-entry(
  name: "add",
  signature: "add(expr, i=a..b)",
  description: [
    Sum `expr` as `i` ranges from `a` to `b`. The iteration
    variable `i` is locally scoped. An empty range (where
    $a > b$) returns 0.
    #index-main[add]
  ],
  params: (
    ([expr], [Any], [Expression to evaluate at each step]),
    ([i=a..b], [Range], [Iteration variable and bounds]),
  ),
  examples: (
    ("add(i^2, i=1..5)", "55"),
    ("add(i, i=1..0)", "0"),
  ),
  related: ("mul", "seq"),
)

#func-entry(
  name: "mul",
  signature: "mul(expr, i=a..b)",
  description: [
    Multiply `expr` as `i` ranges from `a` to `b`. The iteration
    variable `i` is locally scoped. An empty range returns 1.
    #index-main[mul]
  ],
  params: (
    ([expr], [Any], [Expression to evaluate at each step]),
    ([i=a..b], [Range], [Iteration variable and bounds]),
  ),
  examples: (
    ("mul(i, i=1..5)", "120"),
    ("mul(i, i=1..0)", "1"),
  ),
  related: ("add", "seq"),
)

#func-entry(
  name: "seq",
  signature: "seq(expr, i=a..b)",
  description: [
    Generate a list by evaluating `expr` as `i` ranges from `a`
    to `b`. The iteration variable `i` is locally scoped. An
    empty range returns an empty list `[]`.
    #index-main[seq]
  ],
  params: (
    ([expr], [Any], [Expression to evaluate at each step]),
    ([i=a..b], [Range], [Iteration variable and bounds]),
  ),
  examples: (
    ("seq(i^2, i=1..5)", "[1, 4, 9, 16, 25]"),
    ("seq(2^k, k=0..4)", "[1, 2, 4, 8, 16]"),
    ("seq(i, i=1..0)", "[]"),
  ),
  related: ("add", "mul"),
)

=== Variable Management

==== anames
#index-main[anames]

The `anames()` function returns a sorted list of all currently
assigned variable names.

#repl-block("q> x := 1: y := 2:
q> anames()
[x, y]")

==== restart
#index-main[restart]

The `restart()` function clears all variables, procedures, and resets
the session. It returns the string `Restart.`

#repl-block("q> x := 42:
q> restart()
Restart.")
