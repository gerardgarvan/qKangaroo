// main.typ -- q-Kangaroo Reference Manual master document
//
// Compile with: typst compile manual/main.typ manual/q-kangaroo-manual.pdf

#import "@preview/in-dexter:0.7.2": *
#import "template.typ": *

// ---------------------------------------------------------------------------
// Document metadata
// ---------------------------------------------------------------------------

#set document(title: "q-Kangaroo Reference Manual", author: "q-Kangaroo Contributors")

// ---------------------------------------------------------------------------
// Page setup
// ---------------------------------------------------------------------------

#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "New Computer Modern", size: 11pt)
#set heading(numbering: "1.1")
#set par(justify: true)

// Chapter headings (level 1) start on a new page
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(2em)
  text(size: 20pt, weight: "bold", it)
  v(1em)
}

// ---------------------------------------------------------------------------
// Title page
// ---------------------------------------------------------------------------

#include "chapters/00-title.typ"

// ---------------------------------------------------------------------------
// Table of contents
// ---------------------------------------------------------------------------

#outline(depth: 3, indent: auto)
#pagebreak()

// ---------------------------------------------------------------------------
// Chapters
// ---------------------------------------------------------------------------

#include "chapters/01-quick-start.typ"
#include "chapters/02-installation.typ"
#include "chapters/03-cli-usage.typ"
#include "chapters/04-expression-language.typ"
#include "chapters/05-products.typ"
#include "chapters/06-partitions.typ"
#include "chapters/07-theta.typ"
#include "chapters/08-series-analysis.typ"
#include "chapters/09-relations.typ"
#include "chapters/10-hypergeometric.typ"
#include "chapters/11-mock-theta-bailey.typ"
#include "chapters/12-identity-proving.typ"
#include "chapters/13-worked-examples.typ"
#include "chapters/14-maple-migration.typ"
#include "chapters/15-appendix.typ"

// Back-of-book index is generated inside chapters/15-appendix.typ
