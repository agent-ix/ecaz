# Task 205 review request: Algorithm 1 contract

This packet requests review of the FR-079/FR-081 contract slice for ec_distann
expansion pushdown. It defines the candidate limit `l`, owner-side
prune/sort/truncate behavior, and coordinator-derived threshold `t`, together
with the conditions under which the result-equivalence argument holds.

The source changes were bundled into `d27e2fdde`; the bundling note is the
durable attribution record. The ABI follow-up is `615fd72b2`.

Please review `artifacts/equivalence-argument.md`, especially the interaction
of the remaining expansion budget with `L`, early exit, ties, and tombstones.
This contract packet is open pending outside review.
