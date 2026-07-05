# Review Request: C1 ADR-030 V2 PqFastScan Index Rerank Runtime Visibility

Current head: `41fa577`

## Context

Two recent packets left one debug-surface gap open:

- packet `404` changed source-backed `pq_fastscan` to default rerank through
  `heap_f32`
- packet `408` added an index-aware runtime helper for traversal-mode fallback

Reviewer feedback on both packets pointed at the same remaining problem:

1. the global helper still reports nominal rerank defaults
2. the index-aware helper only resolved traversal mode
3. rerank mode still looked global even when the actual index/runtime lane was
   more specific

That meant `tqhnsw_debug_pq_fastscan_runtime_settings_for_index(...)` could
explain why one index used `binary` vs `pq` traversal, but it still could not
tell the operator:

- whether that index was effectively on `heap_f32` vs `quantized` rerank
- whether the rerank choice came from env override or `build_source_column`
- which source column the index would actually use when heap rerank was active

## Problem

Before this slice, the index-aware helper still copied these rerank fields from
the global helper:

- `pq_fastscan_rerank_mode`
- `pq_fastscan_rerank_source_column`

That was inaccurate in the same way packet `408` fixed traversal:

1. source-backed indexes would always look like the nominal global default
2. quantized env override still left the source-column field looking populated
3. heap-source override through `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`
   was not surfaced as an effective per-index choice

The scan path itself already had the right behavior. The missing piece was
structured visibility into the actual rerank decision.

## Planned Slice

Add one shared rerank-decision seam and route both the scan setup path and the
index-aware debug helper through it:

1. factor `pq_fastscan` rerank selection into an explicit decision object
2. keep scan behavior unchanged
3. extend `tqhnsw_debug_pq_fastscan_runtime_settings_for_index(...)` with:
   - effective rerank mode
   - rerank-mode resolution reason
   - effective rerank source column name, only when heap rerank is active
4. add coverage for:
   - source-backed default heap rerank
   - explicit `quantized` override
   - explicit heap rerank with `source_raw` override

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. in `src/am/scan.rs`
   - added `PqFastScanRerankModeResolution`
   - added `PqFastScanRerankModeDecision`
   - factored rerank selection into
     `resolve_grouped_rerank_mode_decision(...)`
   - added exported debug/test seam
     `resolve_pq_fastscan_rerank_mode_decision(...)`
   - routed `configure_grouped_heap_rerank_state(...)` through the same
     decision object so the runtime helper and scan setup share one source of
     truth
   - added pure unit coverage for source-backed vs source-less default rerank
     resolution
2. in `src/am/mod.rs`
   - re-exported the rerank decision helper for the debug/test surface
3. in `src/lib.rs`
   - extended
     `tqhnsw_debug_pq_fastscan_runtime_settings_for_index(index_oid)` with
     `pq_fastscan_rerank_mode_resolution`
   - changed the per-index helper to report:
     - effective rerank mode
     - effective rerank resolution
     - effective rerank source column name
   - added pg coverage proving:
     - source-backed fixtures report
       `heap_f32 + default_heap_f32_with_build_source_column + source`
     - explicit `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=quantized` reports
       `quantized + env_override + NULL source column`
     - explicit `heap_f32` plus
       `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN=source_raw` reports
       `heap_f32 + env_override + source_raw`

## Validation

Passed:

- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as earlier packets on this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remained in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This closes the rerank-side visibility gap left open by packets `404` and
`408`:

1. traversal and rerank now follow the same “decision object + resolution
   reason” pattern
2. the index-aware helper now reports the effective rerank lane instead of the
   nominal global default
3. operators can now see whether heap rerank came from:
   - the persisted `build_source_column` default
   - an explicit env override
   - or no heap source at all because quantized rerank is active

## Next Slice

Take the next small reviewer-facing follow-up rather than broad cleanup:

1. add explicit parity coverage that the default source-backed rerank path
   matches explicit `heap_f32`
2. or tighten landing-proof / packet honesty around the still-unexecuted pg
   tests before merge review
