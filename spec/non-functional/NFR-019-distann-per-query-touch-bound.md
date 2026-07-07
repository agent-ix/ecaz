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
| records expanded per query (per-query MAX across the cell, not mean) | < BW×H (early exit) | ≤ BW×H per attempt (restart per FR-082 resets accounting; max 2 attempts) | pipeline-step counter assertion, every cell |
| expanded-count ratio 100k ÷ 10k at fixed BW,H | ≈ 1.0 | ≤ 1.1 | bench counter comparison across scales |
| minimum BW×H achieving distinct_recall@10 ≥ 0.999, per scale | flat across scales | ≤ 2× growth 10k→100k | gate-packet analysis row (guards against the budget-needed-for-recall growth failure mode that killed the partitioned lane) |

## Verification

The pipeline bench step emits per-query expansion counters; the suite
asserts the cap per cell, and the cross-scale ratio row appears in the gate
packet manifest. Any breach fails the run.

## Dependencies

- **Upstream**: [FR-081](../functional/index/distann/FR-081-distann-query-orchestration.md),
  [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
