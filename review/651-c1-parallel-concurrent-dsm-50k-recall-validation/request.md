# Review Request: Parallel Concurrent DSM 50k Recall Validation

## Summary

This packet extends packet 650's recall validation to the same 50k corpus scale
used by packet 648's build-speed measurement.

The packet compares two indexes on the same shared synthetic external-recall
fixture:

- serial-built index: `ec_hnsw.enable_parallel_build_concurrent_dsm = off`,
  `max_parallel_maintenance_workers = 0`
- concurrent-DSM-built index: `ec_hnsw.enable_parallel_build_concurrent_dsm = on`,
  `max_parallel_maintenance_workers = 4`

Fixture:

- PostgreSQL 18.3
- 50,000 corpus rows x 64 dimensions
- 50 query rows x 64 dimensions
- `ecvector` column encoded with `encode_to_ecvector(source, 4, 42)`
- default TurboQuant current-format index
- `m = 6`, `ef_construction = 40`
- recall measured with existing `tests.ec_hnsw_graph_scan_recall_external_summary`
  at `ef_search = 128`

Artifacts:

- `artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.sql`
- `artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.log`
- `artifacts/manifest.md`

## Result

| Build Path | Workers | Build Wall | Graph Phase | Recall@10 ef=128 | Recall@100 ef=128 | Index Bytes |
|---|---:|---:|---:|---:|---:|---:|
| Serial | 0 | 29,128 ms | 27,278 ms | 0.088 | 0.2444 | 11,616,256 |
| Concurrent DSM | 4 | 11,686 ms | 10,577 ms | 0.154 | 0.3752 | 11,616,256 |

At `ef_search = 128`, concurrent DSM recall was higher than serial on this 50k
fixture:

- recall@10 delta: `+0.066`
- recall@100 delta: `+0.13080001`

The exact quantized recall baseline was `1.0` for both indexes, and both
indexes reported `49982` index tuples and identical relation size.

## Interpretation

This larger synthetic validation again does not show a recall regression from
concurrent insertion order. It also reproduces the speed result at 50k scale:
the concurrent DSM graph phase was `10.577s` versus the serial graph phase at
`27.278s`.

The absolute recall values are low for `m = 6`, `ef_construction = 40`,
`ef_search = 128` on this synthetic fixture, so this packet should not be read
as a production-quality recall claim. The relevant comparison is serial-built
versus concurrent-DSM-built topology on the same data and same scan settings.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.sql --log-output review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.log`

## Review Focus

- Confirm this packet extends packet 650 to the packet-648 50k scale without
  changing runtime code.
- Confirm the same corpus/query tables were used for both index builds.
- Confirm this is enough synthetic validation to continue implementation while
  leaving real-corpus recall as the later default-change gate.
