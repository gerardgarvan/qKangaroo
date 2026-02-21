# Phase 46: Documentation - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Document all v3.0 features (scripting language, expression operations, polynomial operations, bivariate series) in the PDF manual, REPL help system, and worked examples reproducing Garvan's qmaple.pdf tutorial. No new functionality — documentation only.

</domain>

<decisions>
## Implementation Decisions

### Manual chapter structure
- Organize by workflow progression: simple → complex (variables → loops → conditionals → procedures → series ops → products)
- Tutorial + reference hybrid: brief explanatory text for each concept, then syntax and examples
- Chapter count is Claude's discretion — split or combine based on length and logical grouping
- Bivariate series (tripleprod/quinprod/winquist with symbolic z) documented inline with product functions, not as a separate section

### Worked example selection
- Exactly 3 worked examples from qmaple.pdf
- One demonstrating for-loops with series computation
- One demonstrating procedure definitions with memoization
- One demonstrating tripleprod bivariate product identity
- q-Kangaroo code only (no Maple original), but cite the specific qmaple.pdf section each example reproduces
- Woven into the chapter near the features they demonstrate, not in a separate section at the end

### Help entry style
- Only add missing help entries: `help for`, `help proc`, `help if` — leave existing series/factor/subs help entries alone
- Short syntax summary + 1 example per entry (match existing help entry style)
- Add "See also:" cross-references linking related topics at the end of each entry
- Check and add missing keywords to tab completion (for, proc, if, elif, else, fi, od, end, local, RETURN) — only add what's not already completable

### Example presentation
- Match existing manual format for code blocks (Typst raw blocks with input + output)
- Minimal commentary — let code speak for itself with syntax description above and example below
- Include a brief "Notes" subsection covering key differences from Maple where genuinely needed
- Cite specific qmaple.pdf section numbers for worked examples so readers can compare

### Claude's Discretion
- Whether to use one chapter or two (based on content length and logical grouping)
- Exact ordering of subsections within the workflow progression
- Which specific qmaple.pdf sections to cite for the 3 worked examples
- Whether any feature needs a "Notes" tip and what to include
- Loading skeleton / formatting details in Typst

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches matching the existing manual style.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 46-documentation*
*Context gathered: 2026-02-21*
