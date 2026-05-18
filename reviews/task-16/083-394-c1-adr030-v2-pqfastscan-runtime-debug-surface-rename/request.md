# Review Request: C1 ADR-030 V2 PqFastScan Runtime Debug Surface Rename

## Context

Packet 392 renamed the runtime env surface from the old
`TQVECTOR_EXPERIMENTAL_ADR030_V2_*` names to canonical
`TQVECTOR_PQ_FASTSCAN_*` names, while keeping the old names as compatibility
aliases.

But one visible debug helper still leaked the old feasibility naming:

- `tqhnsw_debug_adr030_runtime_settings()`
- `grouped_*` column names

That made the debug SQL surface lag behind the runtime env surface it was
already reporting.

## Problem

Without this slice, the codebase still told users:

- use `TQVECTOR_PQ_FASTSCAN_*` runtime knobs

but the debug helper still surfaced them through:

- `tests.tqhnsw_debug_adr030_runtime_settings()`
- `grouped_scan_window`
- `grouped_exact_traversal_scope`
- and similar old names

That is inconsistent and keeps ADR030 branch history visible in the current
first-class `PqFastScan` surface.

## Planned Slice

One compatibility-focused cleanup checkpoint:

1. add a canonical `tqhnsw_debug_pq_fastscan_runtime_settings()` helper
2. keep the old ADR030 helper as an alias
3. update the existing runtime-settings pg test to exercise the canonical
   helper and canonical env names

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Introduced a shared runtime-settings helper struct

Added:

- `PqFastScanRuntimeSettings`
- `current_pq_fastscan_runtime_settings()`

This centralizes the effective runtime-settings lookup so the canonical helper
and the compatibility alias both report the same values.

### 2. Added canonical debug helper and preserved the ADR030 alias

Added:

- `tqhnsw_debug_pq_fastscan_runtime_settings()`

This new helper exposes canonical column names:

- `pq_fastscan_build_enabled`
- `pq_fastscan_scan_enabled`
- `pq_fastscan_scan_window`
- `pq_fastscan_traversal_score_mode`
- `pq_fastscan_rerank_mode`
- `pq_fastscan_rerank_source_column`
- `pq_fastscan_exact_traversal_enabled`
- `pq_fastscan_exact_traversal_scope`
- `pq_fastscan_exact_traversal_strategy`
- `pq_fastscan_exact_traversal_limit`

The existing:

- `tqhnsw_debug_adr030_runtime_settings()`

remains in place as a compatibility alias, but now simply reuses the shared
settings helper and returns the old `grouped_*` column names.

### 3. Updated the pg test to the canonical debug/runtime surface

Updated:

- `test_tqhnsw_debug_runtime_settings_reflect_controls`

It now:

- sets the canonical `TQVECTOR_PQ_FASTSCAN_*` env names
- queries `tests.tqhnsw_debug_pq_fastscan_runtime_settings()`
- asserts against the canonical `pq_fastscan_*` columns

So the regression surface now matches the intended public naming while the old
ADR030 helper remains available.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
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

This is a surface-cleanup checkpoint, not a behavior checkpoint:

1. the canonical SQL debug surface now says `pq_fastscan`, not `adr030` /
   `grouped`
2. the old ADR030 debug helper remains available as a compatibility alias
3. the canonical env names, canonical debug helper, and canonical pg coverage
   now line up

What this slice intentionally does **not** do:

- rename the broader `debug_grouped_*` scan-analysis helpers yet
- remove the legacy ADR030 debug helper
- change any scan runtime behavior

## Next Slice

The remaining cleanup work is mostly around the larger debug/analysis surface:

1. decide whether the remaining `debug_grouped_*` helper names should be
   renamed before merge or left as lower-priority diagnostic debt
2. continue shrinking the remaining old naming leaks that still touch the
   first-class landing surface
