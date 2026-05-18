## Feedback: ADR-030 v2 Frontier-Head Exact Traversal

A narrow, well-scoped negative-result packet. The code is clean;
the experiment lands where the hypothesis said it would; the
conclusion reorients the branch toward the binary-traversal
experiment in 359.

### What's right

- **Strategy surface is explicit and extensible.** `GroupedExactTraversalStrategy`
  enum at `scan.rs:433` with `Expansion` (default) and `FrontierHead`
  variants. Adding future strategies (e.g., a "top-k re-rerank" or
  "beam-expansion exact") is now a local change, not a structural
  one. Right shape.
- **Scope compatibility enforced at resolve time.** `frontier_head`
  is rejected unless `scope = layer0`, because the live visible-frontier
  scheduler only exists on layer-0. That's the right constraint to
  surface as an error at amrescan rather than silently doing nothing
  at higher layers.
- **Replacement on the visible frontier rather than a side buffer.**
  Rescoring updates the visible frontier candidate in place and
  reseeds the scheduler. That means all downstream frontier
  consumers automatically see the exactified score — no risk of two
  parallel orderings drifting. The existing visible-frontier helper
  in `src/am/search.rs` is the right seam to use.
- **Fix for the debug-helper refresh workflow belongs in this
  packet.** `CREATE OR REPLACE FUNCTION` can't change OUT row type
  in place; explicitly dropping before recreating is the standard
  workaround. Landing that fix here means future diagnostic packets
  that add output columns to the runtime settings probe won't
  re-hit this footgun.
- **Negative result named directly.** Lines 196-207: "exactifying
  only the current visible frontier head is too weak to change the
  grouped-v2 candidate set." No hedging. Good.
- **Hot-path profile confirms the work actually ran.** Line 170:
  `grouped exact us = 308.9`, `grouped exact calls = 16.2`,
  `budgeted exact candidates = 0.0`. So the strategy did take the
  exact path and did not accidentally fall through to the budgeted
  expansion path. That's the right sanity check and it's visible in
  the counters 357 added.

### Concerns

1. **~16 exact calls over ~540 candidate score calls is ~3%.** The
   hot-path table shows `grouped exact calls = 16.2-16.3` per query
   at frontier-head mode, on a lane that does ~540-620 candidate
   scores. So frontier-head exactifies ~3% of the candidates. That
   the operation barely moves recall at 3% is not surprising — what
   would have been surprising is if it *had* moved recall at 3%.
   The packet is fine; just worth naming the coverage fraction
   directly in the interpretation so readers see why the result is
   structurally expected.

2. **"Until the current visible best already has an exact cached
   score" is the termination clause** (per line 101 of the request:
   "repeats until the current visible best already has an exact
   cached score"). That's a good stopping condition — it prevents
   infinite loop on a degenerate frontier — but it also means on a
   query where every frontier candidate gets exactified once (rare
   but possible), the strategy is effectively all-layer exact on
   layer-0 only. Worth a test that asserts termination in the
   degenerate case. The packet lists pg coverage for strategy-mode
   settings and profile emission, but not for termination of the
   reseed loop under pathological input.

3. **Same 50k Recall@10 (0.6740) as approximate grouped-PQ at the
   same operating point.** Line 163 says "effectively unchanged."
   Actually identical to 3 decimal places. Combined with the
   identical Spearman (0.1706667), NDCG (0.7754), and graph_below_exact
   (39), this is *bit-exact identical* to packet 357's approximate
   traversal result. That's suspicious enough to call out: either
   the 16 exact calls genuinely moved nothing (plausible at 3%
   coverage and pre-existing misordering) or the strategy didn't
   reach enough queries to matter in the aggregate. The hot-path
   counters confirm it ran, so the former explanation holds, but
   the bit-exactness is strong enough to raise eyebrows — worth a
   sentence acknowledging it in the interpretation.

4. **Runtime settings probe now exposes strategy, but no env
   rejection test for `frontier_head + scope=all`.** Per the
   Planned Slice #3 and the pg coverage lines. The code rejects
   that combo at amrescan, per `resolve_grouped_exact_traversal_strategy`;
   the pg coverage description (line 106-110) mentions "invalid
   strategy env rejection" but not specifically the scope-compat
   rejection. Worth ensuring both rejection cases are covered.
   (Maybe they are; I didn't grep the test names.)

### Observation

The "Open Questions" section at lines 217-227 is the best summary
of where ADR-030 is after this packet:

1. is there a better approximate candidate-selection signal already
   on grouped-v2 pages?
2. does grouped-v2 need a different hot traversal payload entirely?
3. should the next investigation stay scan-time-only, or does the
   build path need to emit additional candidate-selection state?

Packet 359 answers (1) with "yes, the binary sign sidecar" — and
it's already on the grouped-v2 pages today, used only as a
prefilter. That's the cheapest possible way out of the grouped-PQ
scoring quality problem.

### Measurement gap still open

For this packet: none — the negative result is clean and doesn't
need wider bench coverage. The structural enum adds carry forward
as experimental surface for future strategies even if frontier_head
itself never becomes the default.

For the branch as a whole: after 355-358, the entire "cheap exact
rescue around the existing grouped approximate scorer" family is
closed with negative results. The open question is entirely
upstream of rerank now.
