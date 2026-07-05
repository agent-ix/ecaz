# Review Request: C1 ADR-030 V2 PqFastScan Index Runtime Fallback Visibility

Current head: `9e90e29`

## Context

Packet `401` made the default `PqFastScan` runtime lane honest:

- `scan_window = 64`
- traversal score mode defaults to `binary`
- layouts without a persisted binary sidecar fall back to grouped-PQ

Reviewer feedback on `401` still left one operator-facing gap:

- `tqhnsw_debug_pq_fastscan_runtime_settings()` now shows effective global
  defaults, but it still cannot explain whether a specific index actually got
  binary traversal or silently fell back to grouped-PQ because its layout lacks
  the binary sidecar

That concern was correct. The existing helper is global and has no index
context, so it cannot answer a layout-driven question truthfully.

## Problem

Before this slice:

1. the scan path had the right behavior
   - env override wins
   - otherwise use binary if the `PqFastScan` layout advertises a binary
     sidecar
   - otherwise fall back to grouped-PQ
2. but the debug surface could only report:
   - configured / default traversal mode globally
3. it could **not** report:
   - the effective traversal mode for one concrete index
   - why that mode was chosen

That meant an operator debugging a recall shortfall still had to inspect index
metadata separately to tell whether the default binary lane actually applied.

## Planned Slice

Add one shared decision seam and expose it through an index-aware debug helper:

1. factor traversal mode selection into a reusable decision object
2. keep the existing global runtime-settings helper unchanged
3. add a new index-aware helper that reports:
   - effective traversal mode for that index
   - why that mode was chosen
   - the layout binary-word count that drove the fallback/default split
4. add pg coverage for:
   - normal binary default
   - metadata-driven grouped-PQ fallback
   - explicit env override

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. in `src/am/scan.rs`
   - factored traversal selection into
     `resolve_pq_fastscan_traversal_score_mode_decision(...)`
   - added explicit resolution categories:
     - `env_override`
     - `default_binary_with_binary_sidecar`
     - `fallback_grouped_pq_missing_binary_sidecar`
     - `non_pq_fastscan_storage`
   - kept the existing scan path behavior unchanged by having the old
     `resolve_grouped_traversal_score_mode(...)` call through the new decision
     seam
2. in `src/am/mod.rs`
   - re-exported the new decision helper for the pg-test/debug surface
3. in `src/lib.rs`
   - added `tqhnsw_debug_pq_fastscan_runtime_settings_for_index(index_oid)`
   - it reports the existing runtime settings plus:
     - `pq_fastscan_traversal_score_mode_resolution`
     - `pq_fastscan_layout_binary_word_count`
   - it errors if pointed at a non-`pq_fastscan` index
4. added pg coverage proving:
   - a normal `pq_fastscan` runtime fixture reports
     `binary` + `default_binary_with_binary_sidecar`
   - a metadata-edited fixture with the binary-sidecar flag removed reports
     `pq` + `fallback_grouped_pq_missing_binary_sidecar`
   - an explicit `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE=pq` override reports
     `pq` + `env_override` while still surfacing the persisted binary layout

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as the rest of this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This closes the specific debuggability gap from packet `401` without lying
about what the old helper can know:

1. the global helper still reports global runtime defaults and overrides
2. the new index-aware helper reports the effective traversal mode for one
   concrete `pq_fastscan` index
3. that helper now tells the operator whether they are on:
   - the normal binary default lane
   - the grouped-PQ fallback because the binary sidecar is missing
   - an explicit env override

## Next Slice

Take the next small reviewer-facing parity gap rather than more naming work:

1. add insert/vacuum reloption-mismatch pg coverage on top of the packet `403`
   runtime guardrail
2. or address any new outside feedback that lands on packets `404+`
