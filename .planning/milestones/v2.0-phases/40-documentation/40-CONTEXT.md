# Phase 40: Documentation - Context

**Gathered:** 2026-02-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Update all documentation surfaces (PDF reference manual, REPL help text, tab completion, migration guide, Python API docstrings) to reflect v2.0 Maple-compatible signatures. Users can learn the system from any entry point -- manual, REPL help, or migration guide.

</domain>

<decisions>
## Implementation Decisions

### PDF manual updates
- Replace v1.x signatures entirely with Garvan-canonical forms -- legacy signatures are undocumented (still work, just not shown)
- Full formal math definitions for all new functions: LaTeX-quality summation formulas, product representations, convergence notes -- textbook-reference level
- New functions slot into existing categories by domain (theta under Products, checkmult under Analysis, etc.) -- no separate v2.0 section
- Worked examples cross-reference Garvan's actual Maple worksheets where applicable (e.g., "As in qseries Example 3.2") to help researchers map between systems

### Migration guide structure
- Workflow-oriented organization: sections like "Computing eta products", "Finding congruences", "Theta identities" -- task-oriented, not alphabetical
- Two-column table format: left column Maple code, right column Kangaroo code -- compact and scannable
- Lives as a new chapter in the PDF manual (Typst) -- single authoritative source, ships with the binary
- Focus only on remaining differences -- where translation is still needed. Don't enumerate identical syntax (the success story is implicit)

### Help text strategy
- Show Garvan-only signatures in help() -- no legacy forms shown, consistent with manual approach
- 1 example per function -- help is a quick reference, the manual has depth
- help(partition_count) silently redirects to numbpart -- no deprecation notice, no mention of rename
- Add new "Jacobi Products" help category for jac2prod, jac2series, qs2jaccombo; other new functions slot into existing categories

### Claude's Discretion
- Python API docstring update scope (targeted updates vs broader overhaul)
- Tab completion additions (straightforward -- just add new function names)
- Exact wording and tone of help entries
- Migration guide chapter placement within the manual

</decisions>

<specifics>
## Specific Ideas

- Cross-reference Garvan's Maple worksheet examples by number (e.g., "qseries Example 3.2") so researchers can verify against original source
- Migration guide should be practical -- a Maple user looks up "how do I do X?" and sees immediately what to type
- Two-column tables keep Maple and Kangaroo side-by-side for easy scanning

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 40-documentation*
*Context gathered: 2026-02-20*
