# Review Request: Parallel Concurrent DSM Recall Validation

## Summary

This packet validates the recall risk called out in packet 648 feedback:
concurrent graph insertion has nondeterministic insertion order, so speed alone
is not enough before promoting the opt-in path.

The packet compares two indexes on the same shared synthetic external-recall
fixture:

- serial-built index: `ec_hnsw.enable_parallel_build_concurrent_dsm = off`,
  `max_parallel_maintenance_workers = 0`
- concurrent-DSM-built index: `ec_hnsw.enable_parallel_build_concurrent_dsm = on`,
  `max_parallel_maintenance_workers = 4`

Fixture:

- PostgreSQL 18.3
- 10,000 corpus rows x 64 dimensions
- 100 query rows x 64 dimensions
- `ecvector` column encoded with `encode_to_ecvector(source, 4, 42)`
- default TurboQuant current-format index
- `m = 6`, `ef_construction = 40`
- recall measured with existing `tests.ec_hnsw_graph_scan_recall_external_summary`
  and `tests.ec_hnsw_graph_scan_recall_ef_sweep`

Artifacts:

- `artifacts/pg18_parallel_concurrent_dsm_recall_validation.sql`
- `artifacts/pg18_parallel_concurrent_dsm_recall_validation.log`
- `artifacts/manifest.md`

## Result

| Build Path | Workers | Build Wall | Graph Phase | Recall@10 ef=64 | Recall@10 ef=128 | Recall@10 ef=200 | Index Bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| Serial | 0 | 4,940 ms | 4,529 ms | 0.288 | 0.343 | 0.343 | 2,334,720 |
| Concurrent DSM | 4 | 1,897 ms | 1,640 ms | 0.369 | 0.403 | 0.411 | 2,334,720 |

At the primary `ef_search = 128` point, concurrent DSM recall was higher than
serial on this fixture:

- serial graph recall@10: `0.343`
- concurrent DSM graph recall@10: `0.403`
- delta: `+0.060000002`

The exact quantized recall baseline was `1.0` for both indexes, and both
indexes reported `9998` index tuples and identical relation size.

## Interpretation

This packet does not prove concurrent insertion order is universally better.
It does show that the opt-in concurrent DSM graph path does not degrade recall
on this shared 10k synthetic external-recall fixture; in this run it improved
recall across all measured `ef_search` values.

The result is consistent with the architecture expectation from packet 632:
workers insert into one globally connected graph, so nondeterministic insertion
order changes topology but does not produce a disconnected local-partition
graph.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.sql --log-output review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.log`

## Review Focus

- Confirm the serial and concurrent DSM indexes are built on the same corpus
  and query tables.
- Confirm the existing external recall helper is the right proof surface for
  this pre-promotion validation.
- Confirm this closes the first recall-quality concern for the opt-in path,
  while still leaving larger/real-corpus recall validation as a future gate
  before changing the GUC default.
