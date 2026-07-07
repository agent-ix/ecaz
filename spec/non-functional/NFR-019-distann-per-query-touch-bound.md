---
id: NFR-019
title: Distann Per-Query Touch Bound
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-019: Distann Per-Query Touch Bound

## Statement

A top-k scan SHALL expand at most BW × H graph-node records (plus
coordinator-local head-index hits), independent of corpus size. The scan
SHALL report the per-query expanded-record count in EXPLAIN and in the bench
pipeline step.

## Scope

- Applies to: every FR-081 scan, single-node and multinode, at every
  benchmarked scale.
- BW and H are the session GUCs of FR-075; the bound holds for whatever
  values a run configures.

## Rationale

This is the anti-scan-fraction requirement — the predecessor architecture's
failure stated positively. SPIRE at 0.99 recall touched 35.7% of 50k and
78.7% of 100k row-instances (`reviews/task-144/012-release-matrix-decision/`
on branch `task-144-spire-closure-ratio-pruning`); a traversal-budgeted
index must have per-query work that does not grow with corpus size. Keeping
the bound hard (even under convergence early-exit) is what makes the gate
comparison meaningful.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| records expanded per query | < BW×H (early exit) | ≤ BW×H | pipeline-step counter assertion, every cell |
| expanded-count ratio 100k ÷ 10k at fixed BW,H | ≈ 1.0 | ≤ 1.1 | bench counter comparison across scales |

## Verification

The pipeline bench step emits per-query expansion counters; the suite
asserts the cap per cell, and the cross-scale ratio row appears in the gate
packet manifest. Any breach fails the run.

## Dependencies

- **Upstream**: [FR-081](../functional/index/distann/FR-081-distann-query-orchestration.md),
  [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
