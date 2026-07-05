# Review Request: C1 ADR-030 V2 PqFastScan Scan-Analysis Debug Surface Rename

## Context

Packet 394 renamed the runtime-settings debug helper to canonical
`pq_fastscan` terminology, but the broader scan-analysis SQL helpers still
exposed the old feasibility-era naming:

- `tqhnsw_debug_grouped_scan_comparison_rows()`
- `tqhnsw_debug_grouped_scan_comparison_summary()`
- `tqhnsw_debug_grouped_scan_order_drift_summary()`
- `tqhnsw_debug_grouped_scan_windowed_rows()`
- `tqhnsw_debug_grouped_scan_windowed_summary()`
- `grouped_result_count`

That left the higher-value debug surface lagging behind the current
`PqFastScan` naming even though the underlying runtime path is already the
first-class storage format.

## Problem

Without this slice, the current debug SQL surface still tells users to think in
terms of:

- `grouped` scan analysis helpers
- `grouped_result_count`

even though the same branch now consistently refers to the storage format as
`PqFastScan` everywhere else that matters for landing on `main`.

## Planned Slice

One compatibility-focused surface checkpoint:

1. add canonical `tqhnsw_debug_pq_fastscan_scan_*` analysis helpers
2. keep the old `tqhnsw_debug_grouped_scan_*` helpers as compatibility aliases
3. rename the user-facing `grouped_result_count` column to
   `pq_fastscan_result_count` on the canonical helpers
4. move the existing pg tests onto the canonical helper names

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Added shared helper plumbing for the scan-analysis wrappers

Introduced:

- `validate_debug_index(...)`
- `pq_fastscan_scan_order_drift_summary_values(...)`
- `pq_fastscan_scan_windowed_rows_values(...)`
- `pq_fastscan_scan_windowed_summary_values(...)`
- `pq_fastscan_scan_comparison_rows_values(...)`
- `pq_fastscan_scan_comparison_summary_values(...)`

This keeps the canonical wrappers and the compatibility aliases on one shared
source of truth instead of duplicating the AM calls and tuple mapping.

### 2. Added canonical `PqFastScan` SQL helpers and preserved aliases

Added canonical helpers:

- `tqhnsw_debug_pq_fastscan_scan_comparison_rows()`
- `tqhnsw_debug_pq_fastscan_scan_comparison_summary()`
- `tqhnsw_debug_pq_fastscan_scan_order_drift_summary()`
- `tqhnsw_debug_pq_fastscan_scan_windowed_rows()`
- `tqhnsw_debug_pq_fastscan_scan_windowed_summary()`

These canonical helpers rename the visible result-count column to:

- `pq_fastscan_result_count`

The existing:

- `tqhnsw_debug_grouped_scan_*`

wrappers remain in place and now reuse the shared helpers so they stay as
compatibility aliases instead of separate implementations.

### 3. Moved the pg coverage to the canonical helper names

Updated the existing pg tests and SQL queries to use:

- `tests.tqhnsw_debug_pq_fastscan_scan_comparison_rows()`
- `tests.tqhnsw_debug_pq_fastscan_scan_comparison_summary()`
- `tests.tqhnsw_debug_pq_fastscan_scan_order_drift_summary()`
- `tests.tqhnsw_debug_pq_fastscan_scan_windowed_rows()`
- `tests.tqhnsw_debug_pq_fastscan_scan_windowed_summary()`
- `pq_fastscan_result_count`

This means the regression surface now validates the canonical naming while the
old `grouped` wrappers remain available.

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

This is a naming/surface-cleanup checkpoint, not a runtime-behavior change:

1. the scan-analysis SQL surface now has canonical `pq_fastscan` entrypoints
2. the canonical wrappers expose `pq_fastscan_result_count` instead of
   `grouped_result_count`
3. the old `grouped` helper names remain available as compatibility aliases
4. the pg regression surface now exercises the canonical names

What this slice intentionally does **not** do:

- rename the rerank-profile helper yet
- rename the generic scan hot-path profile columns yet
- change any scan-analysis behavior or scoring semantics

## Next Slice

The next cleanup checkpoint should finish the remaining profile/debug naming:

1. add a canonical `tqhnsw_debug_pq_fastscan_rerank_profile()` helper
2. add a canonical `tqhnsw_debug_pq_fastscan_scan_hot_path_profile()` surface
3. move at least one pg regression check onto those canonical profile helpers
