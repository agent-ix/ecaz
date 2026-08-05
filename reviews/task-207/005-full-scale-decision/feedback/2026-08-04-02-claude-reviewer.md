---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 02
---

# Task 207 packet 005 — full-scale decision: disposition ENDORSED, closeout blocked on three items

The headline decision is correct and well-evidenced: with the confound
removed (fixed `build_shards=4`, only `head_construction` toggling) the
partition-union candidate is **not** a stable win — recall 0.9568/0.8997/
0.9128 vs stitched 0.9658/0.9051/0.9090 across 10k/50k/100k — and keeping
`stitched_bfs` as default while retaining `partition_union` as an explicit
experimental arm is the right call. Recording the 100k sign behavior as the
finding rather than averaging it away is exactly what the previous round
asked for. All cited numbers match `run/results.jsonl`; storage is
effectively identical; NFR-021 rows are present and conforming; 50 timed
queries throughout. This deconfounded null result also retroactively
explains the previous round: the earlier "+1.8pt at 50k" was the graph-
topology change, not the head.

Blocking closeout of the task (not the disposition):

1. **P1 — the pre-registered hypothesis was never actually tested.** The
   hypothesis is "recall is bounded by entry coverage, and per-partition
   union is the mechanism that fixes it." End-to-end recall says union
   doesn't fix it — but *membership was never measured* (packet 004
   feedback items 2–3: the owner lane is head-independent and byte-identical
   across arms; overlap@k appears nowhere). So the packet can say "union is
   not the remedy" but cannot yet say whether membership is or is not the
   bound — which is the controlling fact Tasks 181/185 established and this
   task exists to resolve. Compute head-membership and overlap@k offline
   from the already-captured head samples and predictions, and state what
   the membership bound actually looks like under both constructions. If
   membership is still the bound and union doesn't move it, that conclusion
   redirects the remaining head work (185/210) and belongs in this decision.
2. **P2 — carry the packet-004 corrections into the decision record:** the
   activation-marker gap (no marker on the physical path; digest difference
   is the actual attestation) and the effective-seed correction (arms ran
   256/256, not the documented 128/128). The FR-080 text amended this round
   must match what shipped.
3. **P2 — phase 2 of the task (search path) still has no recorded
   disposition.** The A/B ran `persisted_head` (the Vamana search), which is
   fine, but the task requires either restoring the ANN path as production
   or recording why the exact scan is retained — and the promoted
   `training_landmarks_exact` policy's status after this round is stated
   nowhere. One paragraph with a decision (and its owner if deferred) closes
   this; silence does not.

Also note the head-cache LRU introduced this round has an inverted per-index
eviction (`head_cache.rs:145-160`: `push_front` + back-to-front eviction
removes the *newest* entry once a third key appears, keeping two stale
entries forever; no test would catch it since head_cache.rs has none). It
does not affect any committed benchmark (≤1 key per index in all runs) but
should be fixed with a test before this branch merges.
