---
task: 172
packet: 009-nfr021-prerequisite-integrity
role: coder
status: review-requested
head: 1e70ea2842176387a05a30cdfaa8df2170044e98
date: 2026-07-29
---

# Review request: NFR-021 prerequisite integrity

## Requested decision

Please confirm the fail-closed finding in `artifacts/integrity-finding.md`:

> Task 172's final matrix must not run yet. NFR-021's raw per-node growth
> threshold contradicts its own genuine-sharding rule and makes every correct
> fixed-roster owner lane fail as corpus cardinality grows.

This packet does not weaken or amend NFR-021. It reports the contradiction for
Task 208, which explicitly owns the mechanical gate and requires a finding when
the gate cannot be implemented as specified.

## New evidence

Task 204's required arm-fidelity measurement has landed. Task 205's
suite-driven baseline/control/candidate matrix has also landed at
10k/50k/100k.

The Task 205 owner lane proves:

- three physical owners;
- exact/disjoint placement;
- zero non-owner and orphan records;
- no coordinator traversal replica;
- balanced published record counts; and
- maximum-node graph-side share `0.334126` at 100k, approximately one third of
  the three-owner cluster total.

Those are the architectural properties NFR-021's statement and scope say a
genuinely sharded O(N) structure must satisfy.

## Contradiction

NFR-021 also requires:

```text
max single-node bytes at 100k / max single-node bytes at 10k <= 2.0
```

Task 172 fixes the roster at three nodes. With hash-balanced sharding, each
owner stores approximately `N / 3` graph records, so correct per-node graph
state is necessarily O(N). A 10× corpus increase therefore produces
approximately 10× raw bytes per owner.

The Task 205 measurements are:

```text
10k max node graph-side bytes:   25,706,496
100k max node graph-side bytes: 277,372,928
raw growth:                     10.789994x
bytes/global-row growth:         1.078999x
100k max-node/cluster share:     0.334126
```

The raw threshold fails, while the topology and normalized measurements show a
balanced physical shard rather than centralized or replicated O(N) state.

## Consequences

1. Do not run or promote Task 172's final matrix against the literal current
   gate. No fixed-three-node physical owner control can satisfy it as N grows.
2. Treat Task 205's recall/latency/transport observations as descriptive
   evidence. Its NFR-based `do not advance` disposition cannot be a valid
   NFR-022 engineering decision while the admissibility criterion is
   contradictory.
3. Task 206 may be implementation-ready after Task 205's A/B, but it has no
   valid decision-bearing control until Task 208 resolves the gate.
4. Task 208 should reconcile the metric with NFR-021's stated intent before
   Task 172 pre-registers its final benchmark/full-metrics matrix.

## Recommended Task 208 resolution

Retain the architectural prohibition and measure it directly:

- zero non-owner graph records and vectors;
- coordinator-resident O(N) derived bytes equal zero;
- maximum owner share bounded near `1 / roster_size` plus an explicit balance
  tolerance and bounded structures; and
- stable bytes per global corpus row, or another normalized density bound,
  across 10k/50k/100k.

If raw per-node bytes must remain flat, the benchmark must instead scale the
roster with N; that is a different matrix from Task 172's fixed three-node
requirement.

## Validation

Evidence-only audit. Calculations and immutable citations are in
`artifacts/integrity-finding.md` and `artifacts/manifest.md`. No tests,
benchmarks, clusters, or corpus commands were run.
