---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 207 packet 003 — full-scale A/B: REQUEST CHANGES

All cited 50k/100k numbers verify against `run-50k-final/results.jsonl` and
`run-100k-final/results.jsonl`, and the owner-scan abandonment is documented
with its rationale. But the packet stops exactly where the review decision
starts, and it deviates from the task's packet plan without saying so.

1. **P1 — the primary hypothesis fails at 100k and the packet does not say
   so.** Candidate recall 0.9124 vs control 0.9182 (−0.58pt) at the largest
   scale, after +1.80pt at 50k and +0.86pt at 10k. A sign flip at scale in
   the headline metric of a P0-recall task is the finding of the packet, and
   `request.md` presents it as a neutral number with no disposition, no
   discussion, and no hypothesis for the reversal. Combined with the
   confound (packet 001 item 1: `build_shards=4` changes the graph itself)
   and the under-filled candidate head (3,729/4,096), there are three
   entangled explanations and no way to pick one. State the result plainly
   and treat the deconfounded re-run as the required next step.

2. **P1 — the task's packet plan was silently restructured and phase 2 was
   skipped.** Task 207 requires packet `003-search-and-sharding` (phase 2:
   restore the persisted-Vamana head search or *record why the exact scan is
   retained*) before `004-full-scale-decision`. This packet renames 003 to
   full-scale and nowhere addresses phase 2. Note the A/B arms all use
   `seed_strategy=persisted_head` with `head_search_width=128` — i.e. the
   measured configuration is not the promoted `training_landmarks_exact`
   production policy that Task 207's Why-section critiques. State explicitly,
   per arm: which head search path executed, whether the shipped default
   changed, and where that leaves phase 2. If the default silently moved
   from the promoted training policy to persisted-head graph search, that is
   itself an unreviewed production change.

3. **P2 — the latencies are not production-representative and contradict the
   Task 206 packet without comment.** This packet's BW128/H5 100k p50 is
   344.0/337.0 ms; Task 206 packet 002 measured the *same shape, same
   corpus, same fixture* at 198.7 ms. The difference is presumably the
   `distann-head-attribution-benchmark` feature build recorded in the
   provenance (the single-index control is unaffected: 34–36 ms in both
   packets). A ~70% latency inflation from instrumentation means these p50s
   cannot be cited for any latency claim, and cross-packet Pareto reasoning
   is invalid. Quantify the instrumentation overhead (one arm, feature on
   vs off) or re-run the final arms on the uninstrumented release build.

4. **P2 — membership/overlap@k remains unreported at 50k/100k.** The
   owner-scan abandonment note explains why the *exact-scan oracle* was
   impractical, but the gate's membership metric was thereby dropped
   entirely at full scale, leaving end-to-end recall as the only evidence —
   precisely what the task told us is insufficient ("so the mechanism is
   visible even when recall does not move" — and at 100k it moved the wrong
   way). If the exact oracle is too slow, a cheaper membership proxy
   (owner-oracle over the query top-k ground truth already on disk in the
   predictions JSON) should be derived and reported.

5. **P3 — head SHA bookkeeping:** `request.md` says code head `3fb1319af`,
   but the runs' provenance records extension SHA `4e1e88978`. Cite the SHA
   that actually built the extension.

Bottom line for Task 207 overall: phase 1 is not closed in either direction.
The union mechanism is real code, but its effect is unattributed (confound +
under-fill), unexplained at 100k (sign flip), and unmeasured mechanistically
(no membership data at scale). Deconfound, fix the fill, then re-run 10k/50k/
100k on the uninstrumented build before any disposition.
