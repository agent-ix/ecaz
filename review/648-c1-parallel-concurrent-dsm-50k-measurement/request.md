# Review Request: Parallel Concurrent DSM 50k Measurement

## Summary

This packet measures the opt-in concurrent DSM graph assembly path from packet
647 after the timing-accounting fix in packet 649.

Fixture:

- PostgreSQL 18.3
- 50,000 synthetic `ecvector` rows
- 64 dimensions
- default TurboQuant current-format index
- `m = 6`, `ef_construction = 40`
- `maintenance_work_mem = '1GB'`
- parallel paths requested 4 maintenance workers and table `parallel_workers = 4`

Artifacts:

- `artifacts/pg18_parallel_concurrent_dsm_50k_timing.sql`
- `artifacts/pg18_parallel_concurrent_dsm_50k_timing.log`
- `artifacts/manifest.md`

## Result

| Path | Wall Time | Workers | Heap Ingest | Flush Total | Graph | Stage | Write | Index Tuples |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Serial | 29,582 ms | 0 | 1,350 ms | 28,102 ms | 27,704 ms | 264 ms | 120 ms | 49,982 |
| Parallel heap ingest, serial graph | 28,463 ms | 4 | 587 ms | 27,853 ms | 27,479 ms | 262 ms | 98 ms | 49,982 |
| Parallel heap ingest, concurrent DSM graph | 11,420 ms | 4 | 591 ms | 10,805 ms | 10,365 ms | 330 ms | 101 ms | 49,982 |

The concurrent DSM graph path launched 4 graph workers and produced the same
index size as the serial and parallel-serial-graph paths: `11624448` bytes.

## Interpretation

The concurrent DSM graph path reduced wall-clock build time by 61.4% versus
serial build (`29,582 ms -> 11,420 ms`).

The graph phase reduced by 62.6% versus serial build (`27,704 ms -> 10,365 ms`).
Compared to the existing parallel heap-ingest/serial-graph path, the concurrent
DSM graph path reduced wall-clock time by 59.9% (`28,463 ms -> 11,420 ms`).

The timing-accounting correction is visible in this run: `heap_ingest_us` for
the concurrent DSM path is now `590878`, matching the parallel serial-graph
path's `587231` instead of including graph assembly work. The graph, staging,
and write phases are now accounted under `flush_total_us`, matching the serial
builder's phase boundary.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.sql --log-output review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.log`

The source checkpoint used for the corrected timing surface was validated in
packet 649 with:

- `cargo test build_parallel -- --nocapture`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_opt_in`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo test`
- `cargo pgrx test pg18`
- `git diff --check`

## Review Focus

- Confirm the packet-local log supports the summarized wall-clock and phase
  timing numbers.
- Confirm the measurement uses the corrected timing boundary from packet 649.
- Confirm this packet should remain speed-only; recall validation should be a
  separate packet before changing the concurrent DSM GUC default.
