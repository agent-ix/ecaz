## Feedback: ADR-030 v2 Grouped Live Rerank Window Cutover

Read `prefetch_next_grouped_windowed_graph_result` at `scan.rs:1897`,
`buffer_grouped_graph_result_candidate` at `scan.rs:1947`,
`pop_best_buffered_grouped_scan_result` at `scan.rs:1807`, the buffer
fields on `TqScanOpaque` at `scan.rs:3272-3274`, and the revised
`candidate_score_dispatch` at `scan.rs:1346`.

### What's right

- **345 nit closed correctly.** `candidate_score_dispatch` computes
  `grouped_score_context_from_scan_state` once at line 1351 and reuses
  it via `unwrap_or_else(|| panic!)` in the arm at 1354. No double
  compute. The panic branch is unreachable by construction
  (`grouped.is_some()` guard in the match arm). Clean.
- **Buffer capacity is structural, not defensive.** The buffer is a
  fixed-size array `[BufferedGroupedScanResult; ADR030_GROUPED_V2_LIVE_RERANK_WINDOW]`
  on the scan opaque, with a `u8` length. `push_buffered_grouped_scan_result`
  asserts on overflow rather than silently growing. The refill loop at
  1902 uses `< WINDOW` as the stop condition, so overflow is unreachable
  by construction — the assert is a correctness invariant, not a
  runtime guard. Right shape.
- **Tiebreak is load-bearing and stable.** `pop_best_buffered_grouped_scan_result`
  uses `left.comparison_score.unwrap_or(left.approx_score).total_cmp(...)`
  and then `.then_with(|| left.approx_rank_base.cmp(&right.approx_rank_base))`.
  With 4-bit quantization producing frequent ties, tying on
  approximate rank (insertion order into the buffer) matches the
  simulation semantics from packet 350 — a test in this packet asserts
  this invariant directly. No surprise.
- **Baseline sidecars survive the cutover.** `approx_rank_base` is
  captured at *buffer insertion time* (line 1971), not at emission
  time, so it reflects the approximate order the candidate was seen in
  regardless of how the buffer reorders it. `emitted_heap_rows` is
  added to `grouped_live_rerank_next_approx_rank` at insertion too, so
  the rank accounting is monotonic in approximate order even when the
  emission permutation shuffles candidates. That's what makes the
  346-350 diagnostics continue to mean "approximate order" after this
  packet lands.
- **Shift-down remove preserves buffer order.** The `for idx in selected_idx..buffer_len-1`
  loop at line 1828 moves the remaining entries left by one, leaving
  the unselected candidates in their original relative order. The
  trailing slot is zeroed to `BufferedGroupedScanResult::default()` to
  avoid leaving stale `heap_tids` vectors. No leak.
- **Refill semantics match the simulation.** `prefetch_next_grouped_windowed_graph_result`
  refills the buffer to full capacity before popping (line 1902
  `while buffer_len < WINDOW` then `pop_best_buffered_grouped_scan_result`).
  When the frontier drains, the buffer shrinks monotonically — the
  "tail shrinks" behavior flagged in 350 feedback is exactly what this
  code does, and the pg proof test asserts live output matches the
  window=4 simulation row-for-row. Consistency preserved.
- **Dispatch boundary is narrow.** `grouped_live_rerank_enabled` is a
  single `matches!(..., GraphStorageDescriptor::GroupedV2(_))` check
  at line 1773. `prefetch_next_graph_result_from_frontier` branches to
  the windowed path for grouped storage and falls through to the
  existing scalar loop otherwise. Scalar paths unchanged. Good.

### Concerns

1. **Window is a `const usize`, not tunable.** Line 23:
   `const ADR030_GROUPED_V2_LIVE_RERANK_WINDOW: usize = 4;`. This is
   fine for the current cutover — packet 352's operating-point
   investigation will tell us whether 4 is the right choice — but
   before any gate lift, this has to become:
   - a GUC (`tqvector.adr030_grouped_rerank_window`), OR
   - a per-index metadata field (picked at build time based on
     operating point), OR
   - a compile-time const that's intentionally fixed and documented as
     such with measurement evidence.

   Hardcoded 4 is the right move to de-risk the cutover packet, but
   packet 352's 50k numbers (grouped trails scalar on both recall and
   latency) are the first evidence that 4 might be too narrow for
   larger corpora. Don't treat "we picked 4" as closed.

2. **Buffer is allocation-free — confirmed.** I almost flagged a
   per-candidate allocation here. `BufferedGroupedScanResult` holds
   `heap_tids: CachedHeapTids`, which is a fixed-size inline struct
   (`[ItemPointer; HEAPTID_INLINE_CAPACITY]` + `u8` length) at
   `scan.rs:152`. So the whole buffer is `Copy`, `heap_tids:
   element.heaptids` is a trivial field move, and no allocations
   happen on the refill path. This is exactly what you want in a hot
   prefetch loop — mention-worthy only because it's a correct choice
   that's easy to regress when someone tries to support variable
   heaptid counts later. Keep `CachedHeapTids` inline.

3. **`approx_rank_base` uses `i32` with `checked_add`.** Line 1974:
   `.checked_add(emitted_heap_rows).expect("...should remain in i32 range")`.
   That's safe, but it's an ~2B rank cap. For realistic queries
   capped at `hnsw.ef_search * heap_tids_per_element`, this won't
   trigger. Fine.

4. **Two separate frontier-consume timers recorded.** The windowed
   path at line 1905-1912 records `record_frontier_consume_elapsed`,
   and the scalar path at line 1855-1863 does the same. But the
   windowed path also records `record_graph_result_materialize_elapsed`
   inside the refill loop (per buffered candidate, line 1922-1927)
   — so at WINDOW=4 the materialize timer captures 4 candidates'
   work attributed to a single emission. That skews the per-emit
   materialize latency upward by roughly `WINDOW`. If anyone reads
   these counters as "cost of emitting one row", they'll be misled.

   Either (a) record the materialize timer per emission (after the
   pop), or (b) divide by the buffer fill count, or (c) rename the
   counter for the windowed path. I'd pick (c) — rename to
   `record_grouped_buffered_materialize_elapsed` and have the timer
   reflect "cost of refilling the window." Different semantics, not a
   per-emission cost.

5. **No test for refill-then-early-exhaustion.** The pg proof tests
   the full window-4-on-a-real-index case, and the unit tests cover
   `window_size=1` no-op and `window_size >= count`. But there's no
   test for "frontier runs out mid-refill, buffer has 2 entries, pop
   until empty, then the scan terminates cleanly." The shift-down
   removal plus the `buffer_len` underflow-safe arithmetic (using
   `saturating_sub(1)` at line 1828 and `-= 1` at line 1832) protect
   against this, but a pg test that hits `frontier exhausts before
   emitting k=10` would be good coverage. Easy to construct with a
   tiny corpus and a large `ef_search`.

### Observation

This is a clean minimal live cutover. The discipline of capturing
`approx_rank_base` at insertion time (not emission time) is the right
call because it's what keeps packet 346-350 diagnostics meaningful
across the cutover. The pg proof that "live output order matches
window=4 simulation exactly" is the right claim to anchor this packet
on — it says "we didn't add a new reorder semantic, we just turned on
the one we already simulated."

That said, this packet is a *structural* cutover, not an *operating
point* claim. Window=4 is a placeholder picked because "it's the
width at which 346-349 coverage metrics started to saturate." Packet
352 is where that choice gets tested on real data. Read this packet
and packet 352 together — on its own, this packet is half the story.

### Measurement gap still open

The measurement gap is exactly what it was after packet 350: no
corpus-scale recall or latency numbers. Packet 352 fills that gap
partially (and gives a mixed verdict — see that feedback). The
window-choice discussion remains open until we have evidence that
window=4 is *sufficient* at 50k, not merely *stable* at 10k.
