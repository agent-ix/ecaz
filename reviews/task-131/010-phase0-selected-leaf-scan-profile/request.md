# Review Request: Phase 0 Selected-Leaf Scan Profile

## Context

Reviewer feedback in packet 009 directed the task to stop expanding the
heap-side Phase 1 matrix and pivot to the scan-time instrumentation Task 131
actually needs for streaming top-k.

This checkpoint adds a diagnostic worker-side scan profile endpoint for the
same request shape as `ec_spire_remote_search_coordinator_local(...)`:

```sql
ec_spire_remote_search_coordinator_local_scan_profile(
  index_oid,
  requested_epoch,
  query,
  selected_pids,
  top_k,
  consistency_mode
)
```

## Code Under Review

Commit: `1430be474739f1110c34212ac223ccc78687435b`

Changes:

- Adds `SpireSelectedLeafScanProfile` with selected/scanned PID counts,
  candidate row counts, truncation/winner counts, block bound-surface counts,
  score timing counters, and `local_kth_score`.
- Adds `collect_quantized_selected_leaf_scan_profile(...)`, reusing the
  existing selected-leaf scan observer rather than changing query behavior.
- Adds the coordinator wrapper and SQL-facing diagnostic function.
- Adds a pure Rust unit test covering the profile counters on an in-memory
  SPIRE selected-leaf fixture.

## Validation

See `artifacts/manifest.md`.

- `cargo check --lib` passed.
- `cargo test --lib collect_quantized_selected_leaf_scan_profile_reports_scan_counters` passed.

## Review Focus

- Does this endpoint expose the right scan-time foundation for Phase 3
  threshold work without continuing the heap-side Phase 1 path?
- Are the bound-surface counters named conservatively enough? They currently
  report existing leaf-block summary availability/selection/skips as the only
  sound upper-bound surface available to the selected-leaf scan.
- Should the next checkpoint wire this endpoint into `ecaz bench suite`
  profile output, or first add per-remote-node libpq collection for the
  profile endpoint?
