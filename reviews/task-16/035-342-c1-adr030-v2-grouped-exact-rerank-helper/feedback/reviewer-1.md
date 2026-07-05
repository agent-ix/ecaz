## Feedback: ADR-030 v2 Grouped Exact Rerank Helper

Read `score_grouped_rerank_payload_result`, `score_grouped_rerank_payload_from_scan_state`,
and the updated `score_grouped_candidate_context` in `src/am/scan.rs`. Verified at
lines 1170-1220.

### What's right

- `score_grouped_rerank_payload_result` is pure: `ProdQuantizer` + `PreparedQuery` +
  `GroupedScoreRerankPayload` → `f32`. No scan state, no IO. That makes it directly
  testable, which it is: `score_grouped_rerank_payload_result_matches_prod_quantizer_path`.
- The helper calls `ProdQuantizer::score_ip_from_parts` — the same math as the
  scalar exact score path. That means grouped rerank math reuses validated
  scoring code, not a reimplementation. This is the right call.
- `score_grouped_rerank_payload_from_scan_state` extracts `prepared_query` and
  `cached_quantizer` from `opaque`, then delegates. Single responsibility per
  helper.
- The stub in `score_grouped_candidate_context` now actually computes the rerank
  score (assigned to `_score`) before raising the gate error. So the helper is not
  just declared, it is reachable and exercised on every grouped call path.

### Concerns

1. **`_score` discarded.** The helper's whole point is to compute a score, but the
   stub throws it away. That's fine today because the gate error fires
   immediately after, but the `_score` binding is a readability crutch — a future
   reader will wonder what the helper is for. When the scorer packet lands, the
   `_score` assignment goes away and grouped scoring becomes `return
   score_grouped_rerank_payload_from_scan_state(opaque, &payload)` directly. Worth
   checking the next packet cleans this up.

2. **Cold-tuple fetch on every rejected call.** As flagged on packet 340, every
   unsupported grouped call now does disk IO for the cold rerank tuple. That's
   currently test-only territory (runtime gate upstream rejects before
   `score_grouped_candidate_context` is reached by real queries), but confirm.
   If a test exercises this path at scale, there will be unexpected I/O overhead.

3. **Sign of score.** `score_grouped_rerank_payload_result` returns
   `-quantizer.score_ip_from_parts(...)` — the negation flips IP to distance. The
   scalar exact path likely does the same. Worth a one-line comment at line 1175:
   "negate inner product to produce distance, matching scalar exact path." Future
   developers will appreciate it.

### Observation

This packet materially changes the situation: the exact rerank math for grouped-v2
is now implemented and tested against the scalar baseline. The rerank half of the
"binary → grouped → rerank" pipeline is effectively done as a pure function. The
remaining scoring work is the grouped approximate step; that's the next packet
(343 starts on that by sharing the PQ scorer primitive).
