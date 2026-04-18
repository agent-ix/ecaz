# Review Request: C1 ADR-030 V2 Grouped Rerank Comparison Output

## Context

Packet `345` enabled grouped-v2 ordered scans behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- grouped query LUT preparation now loads persisted grouped codebooks during `amrescan`
- grouped candidate traversal now uses the shared grouped-PQ approximate scorer
- the runtime gate still rejects grouped-v2 scans by default

The exact grouped rerank helper from packet `342` still was not exercised on real emitted scan
results.

## Problem

Without an emitted-result comparison seam, the grouped runtime still lacked a practical way to
check approximate-vs-exact behavior on actual scan outputs:

1. the exact rerank helper existed only as a scorer-local helper
2. the scan path did not preserve that exact rerank score alongside emitted grouped results
3. debug/test code could drift onto the next prefetched result instead of the row that was just
   returned

That meant the next measurement packet would still need to wire comparison capture and prove it was
aligned with emitted tuples.

## Planned Slice

Batch the next emitted-result comparison slices together:

1. compute the grouped exact rerank score when a grouped graph result is materialized
2. keep the operator-facing ordered output on the grouped approximate traversal score for now
3. carry the exact rerank comparison score through emitted output state
4. expose a debug/test surface that returns heap tid, emitted score, and exact comparison score
   together
5. add a pg proof that the captured comparison score matches SQL `<#>` for the emitted heap row

This slice intentionally excludes:

- no gate lift
- no output-score cutover to exact rerank
- no grouped linear fallback work
- no new recall/latency measurements yet

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. `materialize_graph_result_candidate(...)` now computes a grouped exact-rerank comparison score
   through the existing cold rerank helper when the emitted result came from grouped-v2 storage
2. `CurrentScanResult` / `ScanResultState` now carry an optional comparison score for the current
   emitted element without changing the existing operator-facing score field
3. `PendingScanOutput` now preserves that optional comparison score across duplicate-drain output
   emission
4. `TqScanOpaque` now tracks the last emitted comparison score so debug readers stay aligned with
   the row that was actually returned even when graph-phase prefetch immediately advances current
   state
5. added `debug_gettuple_scan_heap_tids_with_score_comparisons(...)` and re-exported it through
   `am::`
6. added a pg test that:
   - builds a grouped-v2 index behind the experimental build + scan gates
   - captures emitted heap tids plus grouped exact-rerank comparison scores
   - compares those captured comparison scores against SQL `embedding <#> query` for the same heap
     rows
7. added unit coverage for the comparison-score lifecycle on `ScanResultState`

## Measurements

This packet is still a comparison seam only, so there are no new latency or recall measurements
yet.

Validation results for this checkpoint:

- focused validation:
  - `cargo test scan_result_state_comparison_score_tracks_current_result_lifecycle --lib`: passed
  - `cargo test test_grouped_v2_runtime_captures_exact_rerank_comparison_scores -- --nocapture`:
    passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now captures an exact rerank comparison score on emitted grouped scan results
without changing the current approximate ordered runtime behavior.

What this de-risks:

1. exact grouped rerank is now proven on real emitted scan rows, not just helper-local unit tests
2. emitted-result comparison stays aligned with the returned heap row even across graph-phase
   prefetch refresh
3. the next packet can start measuring approximate-vs-exact divergence on grouped outputs without
   first rebuilding another debug seam

## Next Slice

The next grouped runtime batch should use this comparison seam to start measurement-oriented work:

1. capture approximate-vs-exact divergence on emitted grouped candidates
2. decide whether a narrow rerank window or output-score cutover is safe behind the same gate
3. then begin the first grouped recall / ordering evidence needed for any future gate-lift
