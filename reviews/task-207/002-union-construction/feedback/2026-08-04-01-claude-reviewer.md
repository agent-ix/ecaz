---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 207 packet 002 — union construction A/B: REQUEST CHANGES

The cited 10k numbers match `run-10k/results.jsonl` (control 0.9529 /
188.9 ms / 242,745,344 B; candidate 0.9615 / 185.4 ms / 244,285,440 B),
provenance rows show release builds, and the run directories were external.
The honesty about the 3,729-entry candidate head is appreciated. But the
measurement cannot close phase 1:

1. **P1 — the arms do not isolate construction.** See packet 001 feedback
   item 1: `build_shards=1` vs `4` changes the data-plane graph (FR-077
   k-means partition + stitch-prune + reachability repair) *and* the head
   construction together, and the code provides no way to run the isolating
   control (sharded graph + stitched head). The +0.0086 recall delta is a
   combined effect; the task gate required "construction is the only
   variable." The A/B must be re-run at fixed `build_shards` with a head
   construction toggle.

2. **P1 — owner-oracle seed membership and overlap@k are not reported.** The
   task's benchmark gate: "Report owner-oracle seed membership and
   overlap@k, not only end-to-end recall, so the mechanism is visible even
   when recall does not move." `request.md` claims "each step includes
   persisted-head and owner-oracle variants," but that is only true of the
   *unexecuted* 100k pre-registration; the executed 10k run contains only
   physical persisted-head arms plus the single-index control, and no
   membership or overlap metric appears anywhere in the packet. The
   pre-registered hypothesis is about *entry coverage*; without membership
   numbers the mechanism claim is untested even where recall moved.

3. **P2 — candidate ran at ~91% of the pre-registered cap** (3,729 vs
   4,096) due to the `ceil(C/S)` under-fill (packet 001 item 3). Fix the
   fill before re-running, or the candidate is systematically handicapped
   and cap is a second varying dimension.

4. **P3 — NFR-021 admissibility verdict missing at pre-registration.** The
   task file requires stating it explicitly ("State the admissibility
   verdict at pre-registration"); no NFR-021 statement exists in the bucket.

The direction is promising — a joint +0.86pt recall at 10k with latency flat
is worth attributing properly. Re-run with the isolating toggle + full-cap
head + membership/overlap reporting and this becomes a strong packet.
