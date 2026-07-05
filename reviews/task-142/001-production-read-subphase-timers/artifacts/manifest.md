# Task 142 Packet 001 Artifact Manifest

- Head SHA: `151ca33d4332e22e7943f4542c9d8ad836d727de`
- Task bucket: `reviews/task-142/001-production-read-subphase-timers/`
- Timestamp: `2026-07-04`
- Scope: instrumentation-only checkpoint for Task 142 Phase 0.
- Surface: SPIRE production-read profile metrics and `ecaz bench spire-pipeline` profile aggregation.
- Behavior change: none intended for routing, candidate selection, heap merge, or result ordering.

## Code Under Review

Commit `151ca33d` adds production-read profile timing fields:

- Coordinator planning sub-phases:
  - `manifest_load_elapsed_ms`
  - `leaf_count_elapsed_ms`
  - `route_select_elapsed_ms`
  - `local_heap_elapsed_ms`
- Remote candidate receive decode sub-phase:
  - `candidate_decode_elapsed_ms`
- CLI aggregate/result columns:
  - `manifest_load_p50/p95/p99`
  - `leaf_count_p50/p95/p99`
  - `route_select_p50/p95/p99`
  - `local_heap_p50/p95/p99`
  - `candidate_decode_p50/p95/p99`

## Validation

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-cli-profile-render.log` | `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture` | passed |
| `cargo-test-core-profile-rollup.log` | `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture` | passed |

`git diff --check` was clean before the code commit.

## Notes

An earlier broad `cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture` attempted to build unrelated test binaries after the Cargo cache cleanup and failed with `No space left on device`. The narrower `--lib` command above validates the same core rollup test without compiling unrelated binaries and passed.

This packet does not claim Task 142 closeout. It provides the first required measurement surface for reproducing the nlists-linear planning staircase on the release substrate before caching changes.
