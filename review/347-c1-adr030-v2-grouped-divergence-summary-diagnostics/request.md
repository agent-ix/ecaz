# Review Request: C1 ADR-030 V2 Grouped Divergence Summary Diagnostics

## Context

Packet `346` added an emitted-result comparison seam for grouped-v2 scans behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- grouped-v2 ordered scans still emit the shared grouped-PQ approximate traversal score
- emitted grouped results now also carry an exact rerank comparison score through scan output state
- debug/test code can read `(heap tid, approx score, exact comparison score)` for the row that was
  actually returned

That was enough to prove comparison-score alignment, but it still left measurement work at a very
low-level per-row seam.

## Problem

The next grouped runtime decision is whether a rerank window or eventual output-score cutover is
safe behind the same gate. The raw row seam from packet `346` was not yet convenient for that:

1. it required every follow-up measurement to reimplement the same emitted-row aggregation logic
2. there was no stable SQL-facing summary surface for grouped approximate-vs-exact deltas
3. scalar scans did not yet have a baseline proof that the grouped-only metrics stayed inactive

Without a summary seam, the next measurement packet would still spend time rebuilding the same
score-delta bookkeeping.

## Planned Slice

Batch the next grouped measurement slices together:

1. summarize emitted grouped comparison rows into stable score-delta counters
2. expose that summary through a debug SQL surface
3. prove the grouped SQL summary matches the emitted-row helper exactly
4. prove scalar scans report zero grouped comparison metrics

This slice intentionally excludes:

- no gate lift
- no output-score cutover
- no rerank-window behavior change
- no grouped recall / latency claims yet

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. added `debug_grouped_scan_comparison_summary(...)` in `src/am/scan_debug.rs`
2. the new helper detects grouped-v2 metadata, reuses
   `debug_gettuple_scan_heap_tids_with_score_comparisons(...)`, and returns:
   - emitted result count
   - grouped result count
   - compared result count
   - missing comparison count
   - mean absolute score delta
   - max absolute score delta
   - mean signed score delta
3. scalar-storage scans now return emitted row count plus zeroed grouped comparison metrics from the
   same helper
4. re-exported the helper through `src/am/mod.rs`
5. added `tests.tqhnsw_debug_grouped_scan_comparison_summary(index_oid, query)` in `src/lib.rs`
6. added grouped pg coverage that computes the expected summary directly from emitted comparison
   rows and asserts the SQL wrapper matches that emitted-row aggregation exactly
7. added scalar pg coverage that proves the grouped-only summary metrics stay zero on a normal
   scalar index

## Measurements

This packet remains diagnostic only. It adds a stable measurement surface, but it does not change
runtime behavior or claim any grouped ordering improvements yet.

Validation results for this checkpoint:

- focused validation:
  - `cargo test test_grouped_v2_runtime_comparison_summary_matches_emitted_rows -- --nocapture`:
    passed
  - `cargo test test_scalar_runtime_summary_reports_no_grouped_comparisons -- --nocapture`:
    passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now has a SQL-visible divergence summary for emitted grouped scan results,
without changing the current approximate ordered output.

What this de-risks:

1. follow-up measurement work can query one stable summary seam instead of rebuilding score-delta
   aggregation from per-row debug output
2. grouped-only metrics are now proven inert on scalar runtime paths
3. the next packet can focus on emitted-order drift and rerank-window evidence instead of first
   wiring another summary helper

## Next Slice

The next grouped runtime batch should stay diagnostic, but move one level closer to a rerank-window
decision:

1. expose emitted grouped rows with both approximate order and exact comparison order information
2. quantify how far the emitted approximate order drifts from the exact rerank order
3. use that evidence to choose whether a narrow rerank window is worth prototyping behind the same
   gate
