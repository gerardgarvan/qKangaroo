// template.typ -- shared styling and reusable components for q-Kangaroo manual
//
// Provides:
//   - func-entry: template for function reference entries
//   - repl: single-line REPL example helper
//   - repl-block: multi-line REPL transcript helper
//   - Chapter heading styling via show rules

#let version = "0.9.0"

// ---------------------------------------------------------------------------
// REPL transcript helpers
// ---------------------------------------------------------------------------

/// Render a single REPL example with `q> ` prompt.
///
/// - `input`: the expression typed at the prompt
/// - `output`: the result displayed
#let repl(input, output) = block(
  fill: luma(248), inset: 10pt, radius: 4pt, width: 100%,
)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #text(fill: rgb("#888888"))[q> ]#raw(input)\
  #raw(output)
]

/// Render a multi-line REPL transcript as a single raw block.
///
/// - `content`: raw string with multiple `q> ` lines and outputs
#let repl-block(content) = block(
  fill: luma(248), inset: 10pt, radius: 4pt, width: 100%,
)[
  #set text(font: "DejaVu Sans Mono", size: 9pt)
  #raw(content)
]

// ---------------------------------------------------------------------------
// Function entry template
// ---------------------------------------------------------------------------

/// Render a complete function reference entry.
///
/// Parameters:
///   - name: function name (string)
///   - signature: full signature string (e.g. "aqprod(coeff_num, coeff_den, power, n_or_infinity, order)")
///   - description: content block describing the function
///   - math-def: optional content block with mathematical definition
///   - params: array of (name, type, description) triples
///   - examples: array of (input, output) string pairs
///   - edge-cases: array of strings describing edge cases
///   - related: array of related function name strings
#let func-entry(
  name: none,
  signature: none,
  description: none,
  math-def: none,
  params: (),
  examples: (),
  edge-cases: (),
  related: (),
) = {
  heading(level: 3, name)
  // Index the function name as a main entry (bold page number)
  index-main(name)

  // Signature in monospace block
  block(fill: luma(245), inset: 8pt, radius: 4pt, width: 100%)[
    #raw(signature)
  ]

  // Description
  description

  // Mathematical definition (if provided)
  if math-def != none {
    v(0.3em)
    [*Mathematical Definition*]
    v(0.2em)
    math-def
  }

  // Parameters table
  if params.len() > 0 {
    v(0.3em)
    [*Parameters*]
    v(0.2em)
    table(
      columns: (auto, auto, 1fr),
      inset: 6pt,
      stroke: 0.5pt + luma(180),
      table.header(
        [*Name*], [*Type*], [*Description*],
      ),
      ..params.flatten()
    )
  }

  // Examples in REPL transcript style
  if examples.len() > 0 {
    v(0.3em)
    [*Examples*]
    v(0.2em)
    for ex in examples {
      repl(ex.at(0), ex.at(1))
      v(0.15em)
    }
  }

  // Edge cases
  if edge-cases.len() > 0 {
    v(0.3em)
    [*Edge Cases and Constraints*]
    v(0.2em)
    for ec in edge-cases {
      [- #ec]
    }
  }

  // Related functions
  if related.len() > 0 {
    v(0.3em)
    [*Related:* #related.join(", ")]
  }

  v(0.8em)
}

// ---------------------------------------------------------------------------
// Chapter heading styling
// ---------------------------------------------------------------------------

// Chapters start on a new page with a larger heading
// (Applied via show rules in main.typ)
