# Review Request: C1 ADR-030 V2 Debug Heap-Backed Scan And Vacuum Repair

Current head: `a4ccba9`

This packet covers local uncommitted work on top of that head.

## Context

Packet `404` made source-backed `pq_fastscan` default to `heap_f32` rerank, and
the follow-on runtime/debug packets added more scan-profile and comparison
surfaces on top of that lane.

Those surfaces were still subtly mismatched with the real AM call shape:

1. several debug scan helpers opened only the index relation and called the AM
   entrypoints directly
2. they could reuse a stale active snapshot instead of forcing a fresh latest
   snapshot
3. the debug vacuum helper did not populate `IndexVacuumInfo.heaprel`

That was good enough while the test helpers mostly inspected index-owned state,
but it is no longer honest once source-backed rerank and source-backed vacuum
repair depend on real heap access.

## Problem

Before this slice, the debug/test surfaces were exercising a narrower path than
production:

1. ordered debug scans were not consistently heap-backed
2. debug profiling helpers could run against a stale snapshot boundary
3. debug vacuum repair did not advertise the heap relation even when the index
   format needed heap-backed behavior

So the tests could say “pq_fastscan source-backed behavior is correct” while
still driving helper-only scaffolding that a real scan/vacuum call would not
use.

## Planned Slice

Make the debug/test surfaces follow the real call shape:

1. factor one heap-backed scan begin/end helper around
   `index_open` + `table_open` + `index_beginscan`
2. always acquire and push a fresh registered latest snapshot for those debug
   scan helpers
3. route the ordered-scan/profile/rerank helper family through that shared
   helper
4. populate `IndexVacuumInfo.heaprel` in the debug vacuum helper

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/vacuum.rs`

Concrete changes:

1. added `DebugHeapBackedScan` in `src/am/scan_debug.rs`
2. added `debug_push_latest_snapshot(...)` so the debug helpers always run
   against a fresh registered latest snapshot
3. added `debug_begin_heap_backed_scan(...)` /
   `debug_end_heap_backed_scan(...)`
4. switched these helpers to that shared heap-backed path:
   - `debug_gettuple_scan_heap_tids(...)`
   - `debug_profile_ordered_scan_with_limit(...)`
   - `debug_grouped_rerank_profile(...)`
   - `debug_gettuple_scan_heap_tids_with_scores(...)`
   - `debug_gettuple_scan_heap_tids_with_score_comparisons(...)`
5. updated `debug_profile_ordered_scan_with_heap_fetch(...)` to force a fresh
   latest snapshot instead of opportunistically reusing a stale active one
6. taught `debug_vacuum_remove_heap_tids(...)` in `src/am/vacuum.rs` to:
   - resolve the heap relation from the index
   - open it when present
   - set `info.heaprel`
   - close it on teardown

## Validation

Passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Still failing in this environment:

- `bash scripts/run_pgrx_pg17_test.sh`

Observed failure remains the read-only `cargo pgrx install --test` destination:

- `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
- `Read-only file system (os error 30)`

## Outcome

The debug/test helper family now follows the same heap-backed shape the real
source-backed runtime uses:

1. debug ordered scans run with both heap and index relations open
2. the helpers stop depending on stale snapshot reuse
3. source-backed vacuum repair/debug paths see a real `heaprel`

## Next Slice

Now that the helper surface is honest, align the pg test contracts that were
still assuming the old non-source-backed or non-binary fixture behavior.
