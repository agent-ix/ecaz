# Review Request: Parallel Build Worker Timing Split

## Summary

Please review commit `7d42f55`, which splits the parallel index build debug
timing surface into heap-ingest and graph-assembly worker launch counters.

This is a diagnostic-only checkpoint. It does not change index contents or build
execution behavior.

## Changes

- Keeps `workers_launched` as the aggregate maximum worker count seen across
  build phases.
- Adds `heap_workers_launched` to record workers launched by the parallel heap
  ingestion phase.
- Adds `graph_workers_launched` to record workers launched by the concurrent DSM
  graph assembly phase.
- Exposes both new fields through `tests.ec_hnsw_debug_last_build_timing()` so
  follow-up PG18 scale runs can distinguish heap worker availability from graph
  worker availability.

## Validation

- `git diff --check`
- `cargo test build_parallel --lib`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_uses_workers`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_default`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_can_be_disabled`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Notes

No measurement artifact is attached. This packet requests review of a debug
instrumentation split that will support the next worker-headroom and 990k scale
measurements.
