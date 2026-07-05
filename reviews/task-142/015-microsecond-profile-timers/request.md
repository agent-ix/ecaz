# Task 142 Packet 015: Microsecond Profile Timers

Please review the Task 142 production-read profile timer precision slice.

## Summary

Task 141 feedback flagged that the production-read profile timers were
millisecond-granularity, which is too coarse for the single-digit-ms deltas
Task 142 needs to prove. Commit `e3c0e8b98` changes the SPIRE production-read
profile metrics from integer millisecond counters to integer microsecond
counters.

This is an instrumentation precision change only. It does not change routing,
candidate selection, transport dispatch, heap merge, or result ordering.

## What Changed

- `SpireRemoteProductionReadMetrics` now records elapsed profile counters in
  microseconds.
- `ec_spire_remote_search_production_read_profile()` emits
  `*_elapsed_us` metric names for planning, manifest load, leaf count, route
  select, local heap, transport, decode, merge, and total timing.
- `ecaz bench spire-pipeline` consumes `*_elapsed_us` profile metrics, while
  retaining fallback parsing for historical `*_elapsed_ms` rows.
- The production-read timeline payload-decode column is renamed to
  `payload_decode_elapsed_us`; timeline start/completion/end-to-end elapsed
  columns remain millisecond fields.

## Validation

Packet manifest: `artifacts/manifest.md`.

Passed:

```text
cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture
cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture
cargo test -p ecaz-cli spire_pipeline_renders_production_read_timeline -- --nocapture
git diff --check
```

Logs are under `artifacts/`.

## Review Focus

1. Confirm the profile evidence surface is now microsecond-resolution before
   the Task 142 release A/B matrix.
2. Confirm CLI compatibility with old `*_elapsed_ms` result rows is adequate.
3. Confirm timeline payload-decode unit naming is consistent with the renamed
   SQL column.
