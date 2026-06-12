---
id: FR-030
title: Current HNSW Access Method Surface
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-003"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/US-008"
    type: "implements"
    cardinality: "N:1"
---
# FR-030: Current HNSW Access Method Surface

## Requirement

`ec_hnsw` SHALL remain the default general-purpose ANN access method and SHALL support the current main-branch build, scan, insert, vacuum, planner, diagnostics, parallel-build, storage-format, and compressed-domain scoring surfaces.

## Behavior

1. `ec_hnsw` SHALL support `ecvector` and `tqvector` opclasses.
2. Reloptions SHALL include `m`, `ef_construction`, `ef_search`, `build_source_column`, `rerank_source_column`, and `storage_format`.
3. `ec_hnsw.ef_search` SHALL override relation scan breadth when set.
4. PG18 SHALL expose planner ordering callbacks, tree-height callback, custom EXPLAIN counters, and stats where configured.
5. Eligible PG18 builds SHALL support parallel heap ingestion and concurrent DSM graph assembly, with a diagnostic fallback GUC.
6. HNSW scan scoring SHALL route TurboQuant, QJL, RaBitQ, grouped-PQ codec parity, and exact-score mode surfaces through the shared `QuantCodec` / candidate-batch scoring contract where a batch boundary exists.
7. `ec_hnsw.turboquant_exact_score_mode` SHALL select among `exact`, `full_lut`, `tiled_lut`, and `int8_approx` exact-score strategies for measurement and diagnostics, where `exact` is the raw uncompressed scoring path and the other three are compressed-domain modes.
8. HNSW block-kernel acceptance SHALL disclose frontier batch-width distribution because graph-frontier batches often limit 32-wide kernel coverage.
9. Parallel index scan is not part of the active requirement set.

## Acceptance Criteria

### FR-030-AC-1

`CREATE INDEX ... USING ec_hnsw` succeeds for documented `ecvector` and `tqvector` opclasses.

### FR-030-AC-2

`SET ec_hnsw.ef_search = value` changes the effective scan breadth reported by HNSW diagnostics.

### FR-030-AC-3

On PG18, `EXPLAIN (ecaz)` can emit HNSW scan counters for an HNSW index scan.

### FR-030-AC-4

Parallel HNSW build can be enabled or disabled through the documented diagnostic GUC.

### FR-030-AC-5

HNSW compressed-domain batch scoring emits block-kernel counter rows with
surface `hnsw` and the applicable quant kind when the selected scan path reaches
the shared batch scorer.

### FR-030-AC-6

HNSW exact-mode benchmark evidence reports width buckets and does not claim SVE
or 32-wide kernel wins when the measured `>=32` flush share is below the task
gate.
