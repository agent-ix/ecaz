---
type: ADR
id: ADR-084
title: "SPIRE Closure and Probe-Pruning Distance Surrogate"
status: ACCEPTED
impact: Governs Task 144 closure assignment diagnostics, the future closure_epsilon reloption, and the ec_spire.probe_distance_ratio GUC. Affects SPIRE build routing and scan routing only.
date: 2026-07-05
---
# ADR-084: SPIRE Closure and Probe-Pruning Distance Surrogate

## Context

Task 144 adds two SPANN-shaped controls to SPIRE:

- build-side closure assignment, which replicates a vector into every leaf
  whose centroid distance is within a ratio of the nearest centroid; and
- query-time probe distance-ratio pruning, which keeps only routed leaves close
  enough to the best route.

The existing SPIRE router ranks centroids by plain inner product. It does not
normalize indexed vectors, query vectors, or centroids before routing. Task 144
Phase 0 diagnostics therefore used the available routing score to compute a
distance-like value:

```text
d_route(score) = max(0, 1 - score)
```

Reviewer feedback on `reviews/task-144/002-closure-geometry-simulation/` found
that this is order-preserving for nearest-leaf selection but is not a true
metric. If `score > 1`, the value floors to zero; any multiplicative ratio band
then collapses to exact zero-distance ties.

## Decision

Until a later ADR changes the routing scorer, Task 144 closure assignment and
query-time probe distance-ratio pruning SHALL use the same route-score
surrogate:

```text
d_route(score) = max(0, 1 - score)
```

For closure assignment, `score` is the same inner-product score used to rank a
source vector against candidate leaf centroids. For query-time pruning, `score`
is the final routed leaf score already produced by SPIRE recursive routing.

The controls are default-off:

- `closure_epsilon = 0` or unset means no closure replication beyond the
  existing fixed-count assignment path.
- `ec_spire.probe_distance_ratio = 0` means fixed-count leaf probing with no
  ratio pruning.

Non-zero ratio controls are valid only as measurement and experimental
configuration until the Task 144 matrix proves recall, latency, row-fraction,
and storage behavior on release builds.

## Rationale

Using one surrogate on both sides keeps the closure knob and the query-time
pruning knob semantically aligned. A vector replicated under a particular
route-score band is measured against a query pruning rule expressed in the same
units.

The alternative, normalizing in only one half of the system, would make Phase 0
diagnostics, build replication, and scan pruning incomparable. A full normalized
cosine or L2 routing contract remains possible, but it must update build
routing, scan routing, diagnostics, and benchmark baselines together.

The route-score surrogate also preserves the current nearest-leaf ordering:
sorting by `d_route` ascending is equivalent to sorting by inner-product score
descending except for scores floored to zero.

## Known Limitation

`d_route` is not norm-robust. Scores greater than one produce zero distance,
and a ratio threshold of `best_distance * ratio` collapses when the best
distance is zero. This can make small ratio bands look narrower than a true
metric band, especially for high-norm vectors.

Task 144 packets must label this explicitly. Simulated closure magnitudes are
directional until the release A/B matrix measures a real built closure index
and query-time ratio pruning together.

## Measurement Requirements

Any Task 144 promotion or closeout packet must use `ecaz bench suite` on PG18
release builds and include:

- 10k / 50k / 100k A/B cells;
- recall, latency, percent row-instances scanned, and storage;
- isolated attribution for closure on/off and pruning on/off;
- per-query probed-list distributions; and
- recall tail distributions.

The Phase 3 matrix must run on top of the Task 143 ranking fix. Pre-Task-143
recall cannot be used as closure/pruning evidence.

## Alternatives Considered

### Normalized Cosine Distance Immediately

Deferred. This would address the norm-sensitivity caveat, but the current scan
pruning point only has the final routed leaf score. It does not carry raw
query/centroid vectors through the pruning step. Introducing normalized cosine
for only build-side closure would create two different ratio units.

### Raw Negative Inner Product

Rejected. A multiplicative ratio over negative values is awkward and reverses
intuition near zero. The existing floored `1 - score` proxy is imperfect, but
it is non-negative and already matches Phase 0 diagnostic artifacts.

### Fixed Absolute Score Gap

Rejected for Task 144. A score-gap knob is closer to the existing
`adaptive_nprobe_score_gap` diagnostic and does not model SPANN's ratio-bounded
closure/pruning pair.
