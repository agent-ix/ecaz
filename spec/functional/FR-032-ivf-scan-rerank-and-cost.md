---
id: FR-032
title: IVF Scan, Rerank, and Costing
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: process
relationships:
  - target: "ix://agent-ix/tqvector/US-013"
    type: "implements"
    cardinality: "N:1"
---
# FR-032: IVF Scan, Rerank, and Costing

## Requirement

`ec_ivf` SHALL implement ordered scan behavior over selected posting lists and expose planner/diagnostic surfaces sufficient for local performance tuning.

## Behavior

1. Scans SHALL resolve effective `nprobe` from session GUC, relation reloption, or automatic `ceil(sqrt(nlists))`.
2. `ec_ivf.nprobe` SHALL override relation `nprobe` when set to a positive value.
3. `ec_ivf.rerank_width` SHALL override relation `rerank_width` when set to zero or higher.
4. `heap_f32` rerank SHALL rerank approximate candidates from heap `ecvector` data.
5. PG18 SHALL expose strategy translation and tree-height callback wiring for IVF where supported.
6. IVF cost snapshots SHALL expose planner inputs and modeled cost state.

## Acceptance Criteria

### FR-032-AC-1

An IVF index scan returns ordered heap TIDs for `ORDER BY embedding <#> query LIMIT k`.

### FR-032-AC-2

Session `ec_ivf.nprobe` and `ec_ivf.rerank_width` overrides are reflected in scan/debug output.

### FR-032-AC-3

`EXPLAIN (ecaz)` can report IVF scan counters on PG18.
