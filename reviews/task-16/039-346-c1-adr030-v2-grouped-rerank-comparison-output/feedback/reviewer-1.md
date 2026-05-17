## Feedback: ADR-030 v2 Grouped Rerank Comparison Output

Read `grouped_candidate_rerank_comparison_score` at `scan.rs:1319`,
`CurrentScanResult`/`ScanResultState`/`PendingScanOutput` comparison-score
plumbing, and `debug_gettuple_scan_heap_tids_with_score_comparisons` at
`scan_debug.rs:1605`.

### What's right

- Comparison score computed at materialization, not at dispatch. That means
  every emitted row carries an exact-rerank score; rows that were pruned
  during traversal never pay for cold-tuple fetch. Cost is O(k) where k is the
  emitted count, not O(n).
- Operator-facing ordering stays on the approximate score. This packet is
  purely additive — the comparison is a sidecar, not a behavior change.
- `last_emitted_comparison_score` on `TqScanOpaque` solves the prefetch-
  advance alignment problem: after a row is emitted, a reader can still ask
  "what was the exact score of the thing we just returned" without the graph
  phase having moved on to the next candidate. That's exactly the right
  bookkeeping choice.
- pg-test actually compares the captured comparison score against SQL
  `embedding <#> query` for the same heap row. That's the load-bearing proof
  — the grouped rerank helper is validated against the authoritative vector-
  distance operator.

### Concerns

1. **Cost of comparison on every emitted row.** If a query emits 1000 rows,
   every one triggers a cold-tuple fetch through
   `grouped_candidate_rerank_comparison_score`. For latency measurement
   purposes that skews the grouped-v2 numbers. When this feeds into real
   measurements (347-349 and beyond), make sure you distinguish:
   - grouped approximate scan alone (no comparison)
   - grouped approximate scan + comparison (what exists now)
   - grouped approximate + narrow-window rerank (what comes later)

   Otherwise "grouped-v2 is X% slower than scalar-v1" will include unnecessary
   measurement overhead.

2. **`grouped_candidate_rerank_comparison_score` returns `Option`.** `None`
   means the candidate wasn't grouped. Good. But it also calls
   `pgrx::error!("... requires metadata-aligned cold payload")` on a
   mismatched payload (line 1328). That error fires at emit time, past the
   point where the traversal is done — a diagnostic error here produces
   particularly confusing ops behavior ("query was almost done then exploded").
   Consider surfacing it as a non-fatal diagnostic (logged and skipped, with a
   `missing_comparison` count) since the comparison is advisory. Important
   distinction because 347/348 already rely on the "missing comparison count"
   as a metric.

3. **Comparison score sign.** `grouped_candidate_rerank_comparison_score` ends
   up calling `score_grouped_rerank_payload_from_scan_state` →
   `score_grouped_rerank_payload_result` which returns
   `-quantizer.score_ip_from_parts(...)`. Sign: negative IP = distance.
   The pg-test compares against SQL `embedding <#> query` which is the
   pgvector negative inner product operator. Signs align, ordering aligns.
   Good — but worth a one-line comment on the helper documenting "emitted
   score and comparison score both use distance (= negative IP)" so anyone
   reading 347's mean-signed-delta metric knows what sign means what.

### Observation

This packet converts grouped-v2 from "executes approximately" to "executes
approximately with verifiable ground truth on emitted rows." That's what the
next three diagnostic packets (347-349) ride on. Tight seam, right shape.
