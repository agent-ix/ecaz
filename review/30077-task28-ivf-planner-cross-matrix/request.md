# Task 28 IVF Planner Cross Matrix

## Scope

This packet records the A6 follow-up for Task 28: verify planner behavior after the A1 cost-model repair across non-prepared query shapes, with `ec_ivf` and `ec_hnsw` indexes available on the same column.

Baseline code checkpoint: `727a3fb2795c13be63b2f8c0a2fb89cb50da72bc`.

## Fixture

Local PG18, existing 10k DBPedia-derived table:

- Table: `task28_ivf_postopt10k_n128w25_corpus`
- IVF index: `task28_ivf_postopt10k_n128w25_idx`, size `10166272` bytes
- HNSW index created for this matrix: `task28_ivf_postopt10k_n128w25_hnsw_idx`, size `13664256` bytes
- Primary-key btree index also present
- `ec_ivf.nprobe = 32`
- `ec_hnsw.ef_search = 64`
- `enable_seqscan = on`

## Results

| shape | selected plan | execution time | note |
| --- | --- | ---: | --- |
| Non-prepared KNN `LIMIT 10` | `ec_ivf` index scan | `50.003 ms` | IVF selected over HNSW. |
| Non-prepared KNN `LIMIT 1000` | `ec_ivf` index scan | `54.043 ms` | IVF returned 25 candidates on this surface. |
| Mixed predicate `id <= 1000`, KNN `LIMIT 10` | primary-key scan plus top-N sort | `467.404 ms` | Planner avoided IVF/HNSW; runtime is poor because scoring 1001 heap rows dominates. |
| Non-KNN selective count `id <= 100` | primary-key index-only scan | `0.096 ms` | Correctly did not select IVF. |
| Low-selectivity non-KNN count `id > 0` | sequential scan | `1.779 ms` | Correctly did not select IVF. |

## Interpretation

A6’s broad planner-selection concern is mostly addressed for the tested 10k surface: pure non-prepared KNN chooses IVF with both ANN indexes present, and non-KNN shapes do not choose IVF.

The mixed predicate shape remains a planner-quality concern. The planner chooses the primary-key path because the btree predicate is selective, but runtime is bad because the vector sort still evaluates 1001 source rows. This is not a regression from A1, but it should stay visible for a later mixed-predicate cost/strategy pass.

No DiskANN work is included in this packet.

## Artifacts

- `artifacts/planner_cross_matrix.sql`
- `artifacts/planner_cross_matrix.log`
- `artifacts/manifest.md`
