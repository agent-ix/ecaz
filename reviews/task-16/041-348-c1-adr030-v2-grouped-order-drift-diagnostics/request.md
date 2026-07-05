# Review Request: C1 ADR-030 V2 Grouped Order Drift Diagnostics

## Context

Packet `347` added a SQL-visible grouped divergence summary behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- emitted grouped scan rows already carried an exact rerank comparison score from packet `346`
- grouped scans now expose score-delta summary metrics for emitted rows
- scalar scans prove those grouped-only summary metrics stay zero

That summary is useful for aggregate score drift, but it still does not show how emitted approximate
order compares to exact rerank order on a row-by-row basis.

## Problem

The next runtime question is whether a narrow exact rerank window is worth prototyping behind the
same gate. The summary-only seam from packet `347` was still missing the core row-level evidence
for that decision:

1. it could show that scores differ, but not which emitted rows would move under exact rerank
2. there was no stable SQL surface for approximate rank versus exact comparison rank
3. scalar scans still lacked a baseline proof that those order-drift columns stay inactive

Without a row-level ordering seam, the next rerank-window packet would still need to rebuild its
own emitted-row ranking logic.

## Planned Slice

Batch the next ordering-diagnostic slices together:

1. expose emitted grouped comparison rows with approximate rank and exact rerank rank
2. surface the signed rank shift for each emitted row
3. keep scalar runtime rows on the same debug surface, but leave grouped-only columns null
4. prove the SQL row surface matches emitted-row comparison data exactly

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

1. added `debug_scan_uses_grouped_storage(...)` in `src/am/scan_debug.rs` so grouped-only debug
   helpers share one metadata check
2. added `debug_grouped_scan_comparison_rows(...)`, which:
   - reuses emitted `(heap tid, approx score, exact comparison score)` rows
   - assigns a 1-based emitted approximate rank
   - sorts grouped exact comparison scores to derive a 1-based exact rank
   - reports `exact_rank_shift = approx_rank - exact_rank`
3. scalar scans now return the same emitted rows with grouped-only columns left `NULL`
4. added `tests.tqhnsw_debug_grouped_scan_comparison_rows(index_oid, query)` in `src/lib.rs`
5. added grouped pg coverage that reconstructs exact ranks from emitted comparison rows and proves
   the SQL row surface matches that ranking exactly
6. added scalar pg coverage that proves `comparison_score`, `exact_rank`, and
   `exact_rank_shift` stay null on a normal scalar index

## Measurements

This packet is still diagnostic only. It adds emitted-order drift visibility, but it does not yet
change scan behavior or claim any rerank-window win.

Validation results for this checkpoint:

- focused validation:
  - `cargo test runtime_comparison_rows -- --nocapture`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now exposes row-level emitted-order drift diagnostics for grouped scan results
without changing the current approximate ordered runtime.

What this de-risks:

1. follow-up work can measure which emitted rows move under exact rerank, not just that scores
   differ in aggregate
2. the first exact-best row’s emitted rank is now observable through a stable debug surface
3. the next packet can size a possible rerank window from real emitted-order evidence instead of
   rebuilding row ranking logic again

## Next Slice

The next grouped runtime batch should stay behind the same external gate, but start turning the
diagnostics into a concrete experiment:

1. summarize row-level rank shifts into top-row / small-window evidence
2. choose a narrow rerank-window candidate from that evidence
3. prototype that window behind the experimental grouped scan gate and compare emitted ordering
