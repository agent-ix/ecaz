# Task 61: HNSW Scan Frontier Overhead

- code commit: `7928649b0`
- benchmark packet: `benchmarks/task61-hnsw-scan-frontier-overhead/`
- validation artifacts: `artifacts/manifest.md`

## Summary

This checkpoint starts the Task 61 optimization pass after the AWS Graviton
baseline. It keeps the behavioral surface unchanged and targets two scan-path
constant factors:

- graph prefetch block deduplication now uses a tiny ordered vector instead of
  allocating a `HashSet` for every HNSW neighbor expansion;
- `BeamSearch::forget_queued` now removes from `discovery_order` with a single
  position lookup instead of scanning once to copy and again to retain.

The prefetch helper is covered by a new unit test. Compile-only validation
passes; plain runtime `cargo test` for this pgrx crate fails at process launch
with an unresolved PostgreSQL symbol (`pg_re_throw`), so the runtime signal for
this change is the cloud benchmark packet.

## Validation

- `cargo test -p ecaz --no-run unique_prefetch_blocks_keeps_first_block_order_and_skips_invalid_tids`
  passed; see `artifacts/cargo-test-no-run-unique-prefetch.log`.

## Benchmark Result

Ran only the requested 10k, 50k, and 100k Graviton cells using checked-in
`ecaz bench suite` configs under
`benchmarks/task61-hnsw-scan-frontier-overhead/`.

- 10k and 50k completed and show lower latency at the same recall.
- 100k failed during load with `No space left on device`; the failure is
  recorded in `benchmarks/task61-hnsw-scan-frontier-overhead/artifacts/ssm-100k-failure.json`.
- The `10k-medium` host was cleaned up and paused after the run.
