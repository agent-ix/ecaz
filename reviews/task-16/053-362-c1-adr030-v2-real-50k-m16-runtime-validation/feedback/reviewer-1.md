## Feedback: ADR-030 v2 Real-50k m=16 Runtime Validation

A measurement-only packet. No code diff to review; read as a
data packet landing on top of 361's deterministic-build fix.

### What's right

- **Correct next experiment.** 361 stabilized the m=8 grouped
  surface and reported grouped m=8 ef=128 at 0.910@1.601ms. The
  natural question is whether m=16 — the standard HNSW default
  in most database systems — widens the grouped advantage,
  closes the recall gap to scalar, or both. This packet answers
  that question on the same canonical corpus with the same
  measurement protocol.
- **Runtime settings verified before the rerun.** Per the
  Validation section: grouped build gate + grouped scan gate +
  `grouped_scan_window = 64` + `grouped_scan_score_mode = binary`.
  That's the minimum audit trail 357 established, cleanly
  applied here. Good hygiene.
- **Both lanes built on the same corpus at the same
  `ef_construction=128`.** Apples-to-apples at the build level,
  which is what makes the comparison meaningful given the
  build-determinism fix from 361.
- **Trade-off named honestly.** Lines 118-122: "grouped m=16
  remains materially faster than scalar m=16 / scalar m=16
  remains materially more accurate than grouped m=16." No
  attempt to frame "faster" as "better"; the packet surfaces it
  as a decision-shape change, not a win.

### Concerns

1. **The grouped m=16 ceiling is 0.938, scalar m=16 starts at
   0.944 at ef=40.** That's the load-bearing sentence. For any
   user who wants Recall@10 ≥ 0.944 on this corpus, grouped-v2
   binary-mode at m=16 can't reach it within the measured
   ef≤320 sweep. The frame in the packet is accurate but worth
   emphasizing: at the m=16 operating point, scalar's *cheapest*
   setting already beats grouped's *most expensive* setting on
   recall. That's different from m=8 where grouped's ceiling
   (0.914) was comparable to scalar's (0.890-0.898, depending on
   ef). The story at m=16 is qualitatively different, not just
   quantitatively.

2. **Grouped m=16 ef=40 (0.900@1.155ms) vs grouped m=8 ef=128
   (0.910@1.601ms) is a cleaner headline.** At comparable
   recall, grouped m=16 ef=40 is ~30% faster than grouped m=8
   ef=128. That's the m=16 upside the packet mostly leaves on
   the table — the story is framed as "m=16 still trails
   scalar" but it's also "m=16 beats our own best m=8 operating
   point on latency at similar recall." Worth surfacing as a
   positive m=16 result even though the scalar gap persists.

3. **Exact-quantized ceiling is 0.944 on both lanes.** Lines
   88-93 and 99-106. Both scalar and grouped m=16 share the
   same `0.944` exact-quantized column. That means the
   quantized representation *can* produce a 0.944 ranking; the
   question is whether the candidate-selection signal can reach
   it. Scalar does (it hits 0.944 at ef=40); grouped doesn't
   (tops out at 0.938 by ef=320). So grouped-v2's gap at m=16
   is a *search-time* gap, not a *representation* gap. The
   grouped lane is leaving 0.006 Recall@10 on the table vs its
   own quantization ceiling, while scalar saturates. Worth
   naming explicitly — it reframes the open question from "can
   we quantize better" to "can we traverse the graph smarter."

4. **`ef=64` → `ef=100` → `ef=128` → `ef=160` grouped Recall@10
   all 0.936, then `ef=320` reaches 0.938.** Lines 88-92. The
   grouped lane plateaus hard at 0.936 from ef=64 onward. That
   plateau behavior is actually useful information: additional
   ef-search is not buying recall, so the limiting factor is
   probably the candidate-selection signal quality rather than
   the search breadth. Pairs with concern #3 — the binary-sign
   traversal score is hitting a quality wall on this corpus at
   this density.

5. **No Spearman / NDCG / hot-path counter breakdown.** The
   packet reports Recall@10, exact-quantized Recall@10, and
   mean latency, but omits the Spearman and NDCG columns that
   earlier grouped-PQ / binary-mode packets have been carrying
   (e.g. 357, 359). At m=16 the "within-top-10 ordering
   quality" question is particularly relevant since the binary
   sign signal from 359 was specifically weaker on Spearman
   than scalar. Rerunning with the full measurement surface
   would let readers see whether the m=16 ceiling is
   membership-limited or ordering-limited. Not blocking — it's
   a measurement-only packet and the core finding stands — but
   worth a follow-up if future product conversations hinge on
   "how confident is the grouped-v2 ordering at m=16."

6. **Scalar m=16 ef=128 5.183ms is 3.24x slower than grouped
   m=16 ef=128 2.445ms, but scalar m=16 ef=40 1.903ms is
   1.65x slower than grouped m=16 ef=40 1.155ms.** Both lanes
   have the same `m=16` graph density, so the per-candidate
   cost difference is the candidate-scoring cost. At ef=40 the
   multiplier is 1.65x; at ef=128 it's 3.24x. The multiplier
   grows with ef because scalar's per-candidate cost is higher
   and more candidates means more per-candidate work. That's
   actually a real grouped-v2 win at higher ef — grouped scales
   better with ef than scalar does. Not called out in the
   packet interpretation; worth noting as a sustained positive.

### Observation

The m=8 vs m=16 packet pair (361 then 362) tells a coherent
story:

- at m=8 the grouped and scalar surfaces are close on recall
  (0.910 vs 0.890 at ef=128), and grouped wins on latency by
  ~2x
- at m=16 the grouped surface wins on latency more decisively
  (2-3x depending on ef) but loses on recall (0.938 vs 0.950
  ceiling)

Product framing for the ADR:

> grouped-v2 at m=8 is a "both axes" win at moderate recall.
> grouped-v2 at m=16 is a pure latency lane that trades ~1pt
> of Recall@10 for ~2-3x latency.

The m=16 story is the more useful one commercially — most
deployments default to m=16 and tolerate small recall
concessions for meaningful latency savings. But the m=8 story
is the "cleanest" grouped-v2 result, because it dominates on
both axes. Both worth carrying in the ADR as separate
operating-point recommendations.

### Measurement gap still open

- **Spearman / NDCG / hot-path counters at m=16.** Concern #5.
- **Bigger corpus (100k, 500k) at m=16.** The m=16 ceiling at
  0.938 may reflect 50k-specific density characteristics. A
  bigger corpus could either widen or close the gap; at the
  very least, extrapolation to production-scale ANN recall
  needs a larger-corpus data point.
- **Planner-level SQL timing on the m=16 grouped lane.**
  Packet 363's follow-up. Direct-harness recall/latency is the
  right starting point, but the user-visible latency includes
  SQL operator overhead that isn't measured here.
