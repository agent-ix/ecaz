## Feedback: ADR-030 v2 Grouped Exact Cost Profile

Read this packet as the "packet 356 was wrong, here's why" packet.
The hot-path counter infrastructure it adds is the reason the
correction could be made confidently. That's the real lasting
contribution.

### What's right

- **Closes the 354/356 measurement hygiene gap.** The
  `tests.tqhnsw_debug_adr030_runtime_settings()` probe is exactly
  the settings-audit helper I called for in 354 feedback. Now
  every grouped packet can and should cite it alongside numbers.
- **Hot-path counters that actually separate concerns.** The
  `ScanDebugProfile` extensions split:
  - grouped approx score calls / elapsed
  - grouped exact score calls / elapsed
  - budgeted exact expansions / candidates / exact candidates
  - score-cache hits vs misses

  That's the right granularity. At a glance, readers can now tell
  "how many exact miss cycles are we paying, and is the cache doing
  anything?" That's what the earlier end-to-end latency numbers
  couldn't answer. The answer turns out to be: budgeted exact
  traversal is dominated by exact-miss count, and the score cache
  is functional but small.
- **Shared per-scan score cache reuse for exact scoring.** The
  `exact_score_grouped_candidate_context(...)` helper routes grouped
  exact traversal and grouped emitted comparison through the same
  per-scan cache keyed by element TID. That's what makes "cache
  hits: 207" show up in the all-layer exact row — without it,
  emission-time comparison and traversal-time exact scoring would
  double-compute on the same TID. Right structure.
- **Corrects 356 explicitly in the body of the packet.** Line
  201-203: "This does not reproduce packet 356's earlier `budget = 1`
  parity claim on the verified 50k lane. Treat that earlier
  measurement claim as superseded by this packet." That's the right
  posture — name the supersedence, keep both records, don't pretend
  the earlier conclusion was always known to be wrong. Future
  readers of the branch will thank you.
- **Budget=8 ≈ full exact observation is load-bearing.** Lines
  228-252: at budget=8, exact calls = 394.9 and miss = 394.9 vs
  all-layer exact's 539.4. Budget=8 is ~73% of the exact work with
  matching recall, which is not actually a meaningful budget win —
  it's just "slightly fewer exact calls, same quality." This kills
  the entire per-expansion-budget family as a cost lever, not just
  the budget=1 point. Good of the packet to say so directly in
  lines 251-252.

### Concerns

1. **The corrected budget=1 measurement (0.4900, Spearman -0.17) is
   itself interesting and undersold.** Spearman is *negative* —
   the emitted order is anticorrelated with exact order on this
   lane. At budget=1, the single exact-rescored candidate per
   expansion is being prioritized while the remaining grouped
   approximate candidates land in nearly-random order. That's
   strong evidence that the grouped approximate score is worse than
   random as a *ranking* signal at this operating point, even if
   it's still useful as an *admission* signal. Worth naming in the
   interpretation — "grouped approximate is net-negative as a
   post-rerank ordering signal" is a sharper claim than "budget=1
   destroys quality."

2. **The score cache hit counts are small (16 at approximate, 207
   at full exact, 65-187 for budgets).** With roughly 540 candidate
   score calls on the full exact run, 207 cache hits is ~38%. Hit
   rate depends on what keys show up twice within a scan — emitted
   comparison rescoring of a previously-traversed element, for
   example. Would be useful to know what's populating the cache and
   whether a larger cache footprint would lift that number. Not
   blocking; a follow-on instrumentation packet.

3. **Approximate scoring profile is captured separately from
   `candidate_score_elapsed`.** Per the packet ("kept grouped
   traversal-specific timing separate from the generic
   `candidate_score_elapsed` counter so the new counters explain
   grouped exact work without double-counting"). Right decision for
   grouped paths, but does mean two different counters are in play
   now and a reader has to know which to use. Add a one-line
   docstring on the debug profile struct clarifying that the
   grouped-specific buckets are the authoritative source for
   grouped scans.

4. **Budget=4 at 0.8460 Recall@10 is the "real middle point" but
   it pays 215.9 exact calls vs 539.4 for full exact (40%) for an
   absolute recall of 0.8460 vs 0.8780 (3.2pt gap to full exact,
   12pt gap to the 0.968 NDCG of full exact — wait, Full exact
   NDCG here is 0.9198, budget=4 is 0.8993).** That's not a bad
   latency/quality knee by itself. If 358/359 hadn't superseded
   this family entirely, budget=4 would be the candidate operating
   point. Worth naming explicitly: budget=4 is the only surviving
   budget point that's meaningfully cheaper than full exact with
   modest quality loss, and it's what the family would have to
   beat.

### Observation

This is the packet that finally put the grouped-v2 measurement
surface on firm ground. Prior grouped packets were a mix of good
code and inconsistent hygiene; after this packet, the minimum
measurement protocol is:

1. restart with the wrapper from 356
2. refresh debug helpers
3. run the settings probe → include its output in the request
4. verify `grouped_result_count > 0` on the target index
5. collect hot-path counters alongside external-summary recall

Any future grouped packet that doesn't cite these should be
treated with the same skepticism packet 352 now gets.

### Measurement gap still open

- per-expansion exact budget is closed with a negative verdict.
- the open question, per lines 287-292, is whether an even
  narrower exact-like seam (frontier head, layer-0 only, other
  variants) could produce quality without paying for budget=8's
  near-full cost. Packet 358 answers that: no.
- with both families closed, the real open question becomes
  "is there a better *approximate* traversal score than grouped-PQ?"
  Packet 359 answers that: yes, the binary sidecar.
