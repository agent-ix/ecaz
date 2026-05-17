# Review Request: C1 ADR-030 V2 Grouped Window Evidence Summary

## Context

Packet `348` exposed row-level grouped comparison diagnostics behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- emitted grouped rows already carried both approximate traversal score and exact rerank comparison
  score
- grouped runtime diagnostics now expose emitted approximate rank, exact rerank rank, and signed
  rank shift for each emitted row
- scalar scans already prove those grouped-only rank columns stay inactive

That row surface is enough to inspect single rows, but it is still too low-level for deciding how
deep an exact rerank prefix should be when the grouped runtime path is eventually allowed to reorder
emitted results.

## Problem

The next grouped-v2 runtime decision is whether a narrow exact rerank window is worth prototyping
behind the same gate. Packet `348` still left that question awkward to answer:

1. every follow-up measurement would need to rebuild the same rank-shift aggregation from row data
2. there was no stable summary for whether the exact-best emitted row already appears in a small
   approximate prefix
3. scalar scans still lacked a proof that these grouped-only window metrics stay inert

Without a summarized order-drift seam, the next rerank-window packet would still spend time
recomputing its own evidence.

## Planned Slice

Batch the next measurement slices together:

1. summarize emitted grouped rank drift into stable order metrics
2. expose exact-best small-window evidence for prefix sizes `1`, `2`, `4`, and `8`
3. surface the summary through a debug SQL wrapper
4. prove grouped SQL results match the emitted-row helper exactly
5. prove scalar scans keep the grouped-only window evidence inactive

This slice intentionally excludes:

- no gate lift
- no rerank-window behavior change
- no output-score cutover
- no grouped recall / latency claims yet

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. added `debug_grouped_scan_order_drift_summary(...)` in `src/am/scan_debug.rs`
2. the new helper reuses `debug_grouped_scan_comparison_rows(...)` and returns:
   - emitted result count
   - grouped result count
   - compared result count
   - mean absolute rank shift
   - max absolute rank shift
   - Spearman rank correlation across emitted grouped rows
   - approximate rank of the exact-best emitted row
   - maximum approximate rank among exact top-4 emitted rows
   - boolean evidence for whether the exact-best emitted row is already inside approximate windows
     `1`, `2`, `4`, and `8`
3. scalar-storage scans now return emitted row count plus zeroed or null grouped-only order metrics
   from the same helper
4. re-exported the helper through `src/am/mod.rs`
5. added `tests.tqhnsw_debug_grouped_scan_order_drift_summary(index_oid, query)` in `src/lib.rs`
6. added grouped pg coverage that derives the expected summary directly from emitted comparison rows
   and proves the SQL wrapper matches that aggregation exactly
7. added scalar pg coverage that proves grouped-only order-drift metrics stay inert on a normal
   scalar index

## Measurements

This packet remains diagnostic only. It adds a stable summary for rerank-window evidence, but it
does not yet change emitted ordering or claim a grouped-v2 runtime win.

Validation results for this checkpoint:

- focused validation:
  - `cargo test order_drift_summary -- --nocapture`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now exposes a SQL-visible order-drift summary for emitted grouped scan results
without changing the current approximate ordered runtime.

What this de-risks:

1. follow-up rerank-window work can consume one stable summary seam instead of rebuilding rank-drift
   aggregation from row output
2. exact-best small-window coverage is now directly measurable from emitted runtime results
3. the next packet can prototype a narrow rerank prefix behind the same gate using concrete emitted
   order evidence rather than manual row inspection

## Next Slice

The next grouped runtime batch should turn the new evidence into behavior behind the existing
experimental gate:

1. pick a narrow exact rerank prefix/window from the emitted-order evidence
2. rerank only that emitted prefix on grouped-v2 scans behind `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`
3. compare the reranked output against the existing approximate output and exact comparison scores
