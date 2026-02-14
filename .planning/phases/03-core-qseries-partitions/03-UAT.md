---
status: complete
phase: 03-core-qseries-partitions
source: 03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md, 03-04-SUMMARY.md
started: 2026-02-14T12:30:00Z
updated: 2026-02-14T12:40:00Z
---

## Current Test

[testing complete]

## Tests

### 1. q-Pochhammer Symbol (aqprod)
expected: aqprod(a,q,n) handles all 4 order cases correctly -- zero returns 1, positive uses sequential multiply, negative uses shifted inversion, infinite delegates to generator. q-binomial coefficients match known Gaussian polynomials. 15 tests pass.
result: pass

### 2. Named Infinite Products
expected: All 5 named products (etaq, jacprod, tripleprod, quinprod, winquist) compute correct series. etaq(1,1) matches Euler function, jacprod(1,2) matches theta4, tripleprod verified via bilateral series. 13 tests pass.
result: pass

### 3. Theta Functions
expected: theta2, theta3, theta4 produce correct series expansions. theta3 = 1+2q+2q^4+2q^9+..., theta3^2 matches sum-of-two-squares r_2(n) (OEIS A004018) for 30 values. 7 tests pass.
result: pass

### 4. Partition Count (Pentagonal Recurrence)
expected: partition_count computes p(n) via O(n*sqrt(n)) pentagonal recurrence. Verified p(0)..p(20) match OEIS A000041, and p(200)=3972999029388.
result: pass

### 5. Restricted Partition Generating Functions
expected: distinct_parts_gf matches OEIS A000009, odd_parts_gf equals distinct_parts_gf (Euler's theorem), bounded_parts_gf correct for m=3. All verified against OEIS.
result: pass

### 6. Rank and Crank at z=1 Singularity
expected: rank_gf and crank_gf handle removable singularity at z=1 by returning partition_gf directly. Both match partition_gf, and rank equals crank at z=1. z=-1 specialization also tested.
result: pass

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
