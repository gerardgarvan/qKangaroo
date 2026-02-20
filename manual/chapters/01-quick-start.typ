// 01-quick-start.typ -- Quick Start tutorial (2-3 pages)
#import "../template.typ": *

= Quick Start
#index[Quick Start]

This chapter walks you through installing q-Kangaroo, evaluating your first
q-series expression, and writing a short script. For detailed installation
instructions, see @installation.

== Installing q-Kangaroo <installing>
#index[installation]

Download the pre-built binary for your platform from the
#link("https://github.com/q-kangaroo/q-kangaroo/releases")[GitHub Releases]
page. On Linux or macOS, extract the archive and place `q-kangaroo` somewhere
on your `PATH`. On Windows, extract the zip and optionally add the directory
to your `PATH`. See Chapter 2 for full details.

Verify the installation:

#repl("--version", "q-kangaroo 0.9.0")

== Your First Expression

Launch the interactive REPL by running `q-kangaroo` with no arguments. You
will see the ASCII kangaroo banner and a `q>` prompt.

Start with the partition generating function. The coefficient of $q^n$ in
`partition_gf(N)` is $p(n)$, the number of integer partitions of $n$:

#repl("partition_gf(10)",
  "1 + q + 2*q^2 + 3*q^3 + 5*q^4 + 7*q^5 + 11*q^6 + 15*q^7 + 22*q^8 + 30*q^9 + O(q^10)")

The `O(q^10)` term indicates that the series is truncated at order 10. To
compute the exact number of partitions of 100 as an integer:

#repl("numbpart(100)", "190569292")

== Exploring q-Series

Assign a q-Pochhammer product to a variable using `:=`:

#repl-block("q> f := aqprod(q, q, infinity, 20):
q> f
1 - q - q^2 + q^5 + q^7 - q^12 - q^15 + O(q^20)")

The `:` at the end of the first line suppresses output. The variable `f`
now holds $(q; q)_infinity$ truncated to order 20.

Use `%` to reference the last result. Here we analyze the product form of
the series:

#repl-block("q> prodmake(%, q, 10)
{1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1}")

This confirms that $f = product_(k=1)^(infinity) (1 - q^k)$, with exponent 1 for
each factor $(1 - q^k)$.

== Your First Script <first-script>

Create a file called `example.qk` with the following contents:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("# example.qk -- Euler's pentagonal number theorem
f := aqprod(q, q, infinity, 20):
g := partition_gf(20):
f * g", lang: none)
]

Run the script from the command line:

#block(fill: luma(248), inset: 10pt, radius: 4pt, width: 100%)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw("$ q-kangaroo example.qk
1 + O(q^20)", lang: none)
]

The result `1` confirms the identity $(q;q)_infinity dot 1/(q;q)_infinity = 1$.

== Getting Help

At the `q>` prompt, type `help` to see a list of all 89 built-in functions
grouped by category. To see detailed help for a specific function:

#repl-block("q> help aqprod
aqprod(a, q, n) or aqprod(a, q, infinity, T)

  Compute the q-Pochhammer product (a;q)_n where a is a q-monomial, q is the variable, and n is a non-negative integer.
  When n is 'infinity', use aqprod(a, q, infinity, T) with explicit truncation T.

  Example:
    q> aqprod(q^2, q, 5)
    -q^14 - q^12 + q^8 + q^5 - q^3 - q^2 + 1")

From the command line, `q-kangaroo --help` shows available flags and usage.
