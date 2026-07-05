# Task 142 Review Request: Production-Read Subphase Timers

Please review packet 001 for Task 142.

This is the Phase 0 instrumentation checkpoint. It does not implement epoch caching yet. It makes the current production-read profile granular enough to measure the Task 142 overhead sources before changing cache semantics.

## What Changed

Commit `151ca33d` adds timing fields to `SpireRemoteProductionReadMetrics` and the SQL-visible production-read profile row:

- `manifest_load_elapsed_ms`
- `leaf_count_elapsed_ms`
- `route_select_elapsed_ms`
- `local_heap_elapsed_ms`
- `candidate_decode_elapsed_ms`

The `ecaz bench spire-pipeline` profile aggregation now emits p50/p95/p99 columns for those fields, so suite `results.jsonl` can carry the sub-phase timing table that Task 142 needs.

## Validation

Packet manifest: `reviews/task-142/001-production-read-subphase-timers/artifacts/manifest.md`.

Passed:

```text
cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture
cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture
```

Both logs are under `artifacts/`.

## Review Focus

1. Confirm the timer boundaries are useful for Task 142 Phase 0: manifest load, leaf count, route select, local heap, and candidate row decode.
2. Confirm this is instrumentation-only and does not alter routing/candidate/merge behavior.
3. Confirm the CLI aggregation is sufficient for `ecaz bench suite` JSONL/report evidence in the next A/B packet.
