# Review Request: C1 ADR-030 V2 PqFastScan Profile Debug Surface Rename

## Context

Packet 395 renamed the scan-analysis SQL helpers to canonical `pq_fastscan`
names, but the profile-oriented debug surface still leaked the old feasibility
terminology in two places:

- `tqhnsw_debug_grouped_rerank_profile()`
- `grouped_rerank_*` columns

and the generic hot-path profile helper still exposed:

- `grouped_traversal_*` columns

That left the higher-signal runtime-profile surfaces out of step with the
current `PqFastScan` naming.

## Problem

Without this slice, the branch still tells users to think in terms of:

- grouped rerank counters
- grouped traversal counters

even though the runtime knobs, storage reloptions, and broader debug surface
have already moved to canonical `PqFastScan` terminology.

## Planned Slice

One compatibility-focused profile checkpoint:

1. add canonical `tqhnsw_debug_pq_fastscan_rerank_profile()`
2. add canonical `tqhnsw_debug_pq_fastscan_scan_hot_path_profile()`
3. keep the old `tqhnsw_debug_grouped_rerank_profile()` helper as a
   compatibility alias
4. keep the existing generic `tqhnsw_debug_scan_hot_path_profile()` helper in
   place
5. add pg coverage that exercises the canonical profile helpers

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Added shared value helpers for the profile wrappers

Introduced:

- `debug_scan_hot_path_profile_values(...)`
- `pq_fastscan_rerank_profile_values(...)`

This pulls the AM-facing tuple destructuring into shared helper functions so the
canonical wrappers and the compatibility wrappers stay synchronized without
duplicating the underlying profile extraction logic.

### 2. Added canonical profile wrappers

Added:

- `tqhnsw_debug_pq_fastscan_scan_hot_path_profile()`
- `tqhnsw_debug_pq_fastscan_rerank_profile()`

These canonical helpers expose:

- `pq_fastscan_traversal_*` hot-path columns
- `pq_fastscan_rerank_*` rerank columns

### 3. Preserved compatibility surfaces

Kept:

- `tqhnsw_debug_grouped_rerank_profile()`
- `tqhnsw_debug_scan_hot_path_profile()`

The grouped rerank helper now reuses the shared value helper and remains a
compatibility alias. The existing generic scan hot-path helper also now reuses
the shared extraction helper, but its visible `grouped_traversal_*` column
names are preserved for compatibility.

### 4. Added pg coverage for the canonical profile helpers

Added two pg tests that query the canonical SQL helpers directly:

- `test_tqhnsw_debug_pq_fastscan_rerank_profile_sql_surface()`
- `test_tqhnsw_debug_pq_fastscan_scan_hot_path_profile_sql_surface()`

These tests exercise:

- canonical SQL helper names
- canonical `pq_fastscan_*` column names
- canonical `TQVECTOR_PQ_FASTSCAN_*` env names

So the canonical runtime-profile surface now has explicit regression coverage
instead of relying only on direct internal helper calls.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This is a debug-surface cleanup checkpoint, not a runtime behavior change:

1. canonical rerank and hot-path profile helpers now say `pq_fastscan`
2. the canonical wrappers expose `pq_fastscan_traversal_*` and
   `pq_fastscan_rerank_*` columns
3. the old grouped rerank helper remains available as a compatibility alias
4. the canonical profile wrappers now have direct pg coverage

What this slice intentionally does **not** do:

- remove the old grouped rerank helper
- rename the existing generic `tqhnsw_debug_scan_hot_path_profile()` helper
- rename the remaining direct internal test variable names yet
- change any scoring, traversal, or rerank behavior

## Next Slice

The next remaining cleanup is mostly test/runtime-surface debt:

1. move the remaining `PqFastScan` runtime tests off the legacy
   `TQVECTOR_EXPERIMENTAL_ADR030_V2_*` env names
2. keep shrinking the remaining `grouped` naming leaks that are still visible
   in first-class landing paths
