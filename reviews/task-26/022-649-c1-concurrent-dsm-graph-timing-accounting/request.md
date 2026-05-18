# Review Request: Concurrent DSM Graph Timing Accounting

## Summary

This packet covers commit `f15bb72 Account concurrent DSM graph timing in flush`.

The previous concurrent DSM graph worker checkpoint ran graph assembly inside
`try_parallel_build`, so the debug timing surface reported graph work as part of
heap ingest and excluded it from `flush_total_us`. That made the 50k timing
surface misleading even though the build path itself was functional.

This change moves the opt-in concurrent DSM graph assembly call back under the
ambuild flush timing window:

- `heap_ingest_us` now measures only heap ingest, worker drain, and tuple
  ordering/push.
- `flush_total_us` now includes graph assembly, page staging, and page writes.
- `graph_us`, `stage_us`, and `write_us` remain phase-specific sub-timers for
  the flush window.
- `try_parallel_build` again returns only heap-ingest phase metadata.

## Files

- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`

## Validation

- `cargo test build_parallel -- --nocapture`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_opt_in`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo test`
- `cargo pgrx test pg18`
- `git diff --check`

## Review Focus

- Confirm the concurrent DSM graph assembly phase is now accounted under the
  same flush timing boundary as the serial graph builder.
- Confirm `try_parallel_build` no longer owns graph/page-staging output, which
  keeps heap-ingest phase accounting narrow.
- Confirm the fallback behavior remains unchanged when the concurrent DSM graph
  path is not selected.
