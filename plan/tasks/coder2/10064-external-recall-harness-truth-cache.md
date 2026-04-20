# Review Request: External Recall Harness — Truth Cache

## Context

Prior real-corpus packets:
- `review/224-a4-real-50k-directional-summary/request.md`

The real `50k` external recall harness is currently too expensive to run over
wider query slices interactively. The `224` directional packet calls this out
explicitly: wider slices cost "several minutes of fp32 truth work each time",
and the attempted 25-query four-config gate rerun lost its client session before
completing.

This is a harness architecture problem, not a recall problem.

## Problem

`build_external_recall_context` in `src/lib.rs` bundles three distinct costs
into a single blocking call:

1. **Corpus load** — reads all 50k rows (`source real[]`) from the corpus table
   into a Rust `Vec<Vec<f32>>`. At 50k × 1536 floats this is ~300 MB of heap
   allocation and a full sequential table scan.

2. **fp32 ground-truth computation** — iterates every (query, corpus) pair to
   compute dot-product truth. For Q queries at 50k corpus: Q × 50k inner
   products, computed in Rust. For Q=50 this is 2.5M inner products — fast in
   isolation, but coupled with the corpus load above.

3. **Exact-quantized sequential scans** — issues Q separate SPI calls of the
   form `ORDER BY embedding <#> $1 LIMIT 10` against the corpus table. Each is
   a full sequential scan of 50k `tqvector` rows. For Q=50 that is 50 sequential
   scans, and each carries SPI round-trip overhead.

The four-config gate report (`run_graph_scan_recall_gate_from_external`) calls
`build_external_recall_context` once and then runs four probe configs, so the
context cost is paid once per gate call. But every gate call — even a rerun with
a different query slice — rebuilds the context from scratch.

The live graph scan loop inside
`probe_graph_scan_recall_external_summary_for_context` is cheap: it visits Q
queries against a pre-built index using `debug_gettuple_scan_heap_tids`, which
is fast. The expensive part is the context build, not the probe.

## Proposed Fix

Split `build_external_recall_context` into two phases:

### Phase 1 — Cache Build (expensive, done once per corpus+query set)

Add a new SQL-callable function:

```
ec_hnsw_graph_scan_recall_external_cache_build(
    corpus_table  text,
    query_table   text,
    cache_prefix  text
)
```

This function:
- loads the corpus and queries
- computes fp32 ground-truth top-100 for all queries
- runs exact-quantized top-10 for all queries
- materializes results into two new Postgres tables:
  - `<cache_prefix>_truth` — one row per (query_index, rank): columns
    `(query_idx int4, rank int4, corpus_row_idx int8, score float4)`
  - `<cache_prefix>_exact_quantized` — one row per (query_index, rank): columns
    `(query_idx int4, rank int4, corpus_row_idx int8)`

These tables are created with `CREATE TABLE IF NOT EXISTS` and truncated before
repopulation so the function is idempotent.

### Phase 2 — Probe (cheap, done per index config)

Add a cache-backed context loader:

```rust
fn build_external_recall_context_from_cache(
    corpus_table: &str,
    query_table: &str,
    cache_prefix: &str,
) -> ExternalRecallContext
```

This reads `ground_truth_top_k` and `exact_quantized_row_indices_top10` from
the cache tables instead of recomputing them. It still loads the corpus into
memory (needed for `ctid_to_row_index` and the `corpus_ids` map), but skips the
O(Q × N) truth computation and the Q sequential exact-quantized scans.

Add cache-backed variants of the two public probe surfaces:

```
ec_hnsw_graph_scan_recall_external_summary_cached(
    corpus_table  text,
    query_table   text,
    cache_prefix  text,
    index_name    text,
    m             int4,
    ef_search     int4
) → GraphScanRecallExternalSummaryRow

ec_hnsw_graph_scan_recall_external_gate_report_cached(
    corpus_table   text,
    query_table    text,
    cache_prefix   text,
    fixture_prefix text
) → SETOF (m int4, ef_search int4, recall_at_10 float4, gate_recall_at_10 float4, passes_gate bool)
```

These call `build_external_recall_context_from_cache` instead of
`build_external_recall_context`.

## Scope

- `src/lib.rs`
  - `build_external_recall_context` — unchanged; keep existing function for
    callers that do not have a warm cache
  - `build_external_recall_context_from_cache` — new; reads from materialized
    cache tables
  - `ExternalRecallContext` — unchanged struct; the cache-backed builder
    populates the same fields
  - New SQL-callable: `ec_hnsw_graph_scan_recall_external_cache_build`
  - New SQL-callable: `ec_hnsw_graph_scan_recall_external_summary_cached`
  - New SQL-callable: `ec_hnsw_graph_scan_recall_external_gate_report_cached`

## What Is Not in Scope

- Changing the existing non-cached functions. They must remain as-is.
- Corpus load optimization (that is a separate concern).
- Any change to the `ctid_to_row_index` build path — ctids are index-specific
  and must always be read live.

## Expected Outcome

Once `ec_hnsw_graph_scan_recall_external_cache_build` has been run for a given
`(corpus_table, query_table)` pair, subsequent gate-report or summary calls
using the `_cached` variants are cheap: the expensive O(Q × N) truth work does
not repeat. Widening from a 10-query to a 50-query or 200-query slice then costs
a single one-time cache build, not repeated brute-force work on every probe.

This directly unblocks the wider real `50k` sweep described in
`review/224-a4-real-50k-directional-summary`.
