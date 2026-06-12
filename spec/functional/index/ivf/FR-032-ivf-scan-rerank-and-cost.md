---
id: FR-032
title: IVF Scan, Rerank, and Costing
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
---
# FR-032: IVF Scan, Rerank, and Costing

## Description

`ec_ivf` SHALL implement ordered scan behavior over selected posting lists and expose planner/diagnostic surfaces sufficient for local performance tuning.

## Behavior

1. Scans SHALL resolve effective `nprobe` from session GUC, relation reloption, or automatic `ceil(sqrt(nlists))`.
2. `ec_ivf.nprobe` SHALL override relation `nprobe` when set to a positive value.
3. `ec_ivf.rerank_width` SHALL override relation `rerank_width` when set to zero or higher.
4. `heap_f32` rerank SHALL rerank approximate candidates from heap `ecvector` data.
5. PG18 SHALL expose strategy translation and tree-height callback wiring for IVF where supported.
6. IVF cost snapshots SHALL expose planner inputs and modeled cost state.
7. IVF compressed-domain scan scoring SHALL route TurboQuant, QJL, RaBitQ, and
   grouped-PQ/PqFastScan surfaces through `QuantCodec::score_ip_batch` where
   the selected storage format exposes a batchable search payload.
8. IVF PqFastScan batch scoring SHALL document the interaction between
   scratch-SoA batch decode and suffix-max or cutoff pruning. Batch-on/off MUST
   remain a benchmark axis where pruning tradeoffs affect latency.
9. Non-ORDER-BY SQL over IVF-indexed tables SHALL not fail because the planner
   picked the ANN access method for a plain scan. Until Task 100 lands, this is
   a known robustness gap rather than an accepted behavior.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-032-AC-1 | An IVF index scan returns ordered heap TIDs for `ORDER BY embedding <#> query LIMIT k` | Test |
| FR-032-AC-2 | Session `ec_ivf.nprobe` and `ec_ivf.rerank_width` overrides are reflected in scan/debug output | Test |
| FR-032-AC-3 | `EXPLAIN (ecaz)` can report IVF scan counters on PG18 | Test |
| FR-032-AC-4 | IVF block-kernel benchmark evidence includes `surface=ivf` rows with quant kind, ISA, scalar/kernel counters, and width buckets | Analysis |
| FR-032-AC-5 | Plain non-ORDER-BY statements such as `count(*)` over an IVF-indexed table plan and execute without an ANN scan shape error after the Task 100 planner guard | Test |

### FR-032-AC-1

An IVF index scan returns ordered heap TIDs for `ORDER BY embedding <#> query LIMIT k`.

### FR-032-AC-2

Session `ec_ivf.nprobe` and `ec_ivf.rerank_width` overrides are reflected in scan/debug output.

### FR-032-AC-3

`EXPLAIN (ecaz)` can report IVF scan counters on PG18.

### FR-032-AC-4

IVF block-kernel benchmark evidence includes `surface=ivf` rows with quant kind,
ISA, scalar/kernel counters, and width buckets for claimed compressed-domain
batch-scoring wins.

### FR-032-AC-5

Plain non-ORDER-BY statements such as `count(*)` over an IVF-indexed table plan
and execute without an ANN scan shape error after the Task 100 planner guard is
implemented.

## Dependencies

- **Upstream**: US-013 (implements relationship)
- **Downstream**: none identified
