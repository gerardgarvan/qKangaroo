# Phase 34: Product & Theta Signatures - Context

**Gathered:** 2026-02-19
**Status:** Ready for planning

<domain>
## Phase Boundary

All product and theta functions accept Garvan's exact argument lists so researchers can call them identically to Maple. Covers aqprod, etaq, jacprod, tripleprod, quinprod, winquist, qbin signature updates, plus adding numbpart as primary name for partition counting. Legacy (v1.x) calling conventions continue to work silently.

</domain>

<decisions>
## Implementation Decisions

### Signature coexistence
- Silent, both work: old and new signatures coexist with no deprecation warnings
- Help system shows Maple-style signatures ONLY -- old signatures are undocumented but still functional
- Error messages reference Maple signatures ONLY -- guide users toward Garvan's conventions
- Disambiguation strategy: Claude's discretion per function (cleanest approach for each)

### etaq multi-delta
- Full Garvan support: both `etaq(q, 3, 20)` (single delta) and `etaq(q, [1,2,3], 20)` (multi-delta list) are implemented
- Validation matches Maple behavior -- whatever Maple does for invalid lists, we do the same
- All product functions (tripleprod, quinprod, winquist) match Garvan's exact signatures -- research each function's actual Maple signature and replicate

### numbpart naming
- `numbpart` is the primary name (Maple convention); `partition_count` becomes alias
- `help(partition_count)` redirects to `help(numbpart)` -- one source of truth
- `numbpart` matches full Maple signature -- research what Maple's numbpart actually accepts (overloaded forms)

### Output exactness
- Match Garvan's coefficient ordering exactly (ascending powers as Garvan produces)
- Finite products always expand to polynomials (not product notation)
- O(q^N) notation matches Garvan's exact format -- research and replicate character for character
- Coefficient display matches Garvan -- research whether Garvan uses `2*q^3` or `2q^3` and replicate
- Test against Garvan's actual Maple output (captured test vectors) for coefficient-by-coefficient verification

### Claude's Discretion
- Disambiguation strategy per function: first-arg type, arg count, or other approach as cleanest for each function
- numbpart primary vs equal peers in tab completion ordering
- Exact handling of edge cases not covered by Garvan's documentation

</decisions>

<specifics>
## Specific Ideas

- "Match Garvan exactly" is the overarching principle -- signatures, output format, coefficient ordering, O(...) notation
- "Researchers can visually compare" -- output should look identical to what Garvan's Maple package produces
- Test vectors should be captured from actual Garvan Maple output, not just mathematical definitions

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 34-product-theta-signatures*
*Context gathered: 2026-02-19*
