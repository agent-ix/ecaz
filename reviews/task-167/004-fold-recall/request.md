# Review request — Task 167 M5 packet 004: fold-recall A/B (honest gap)

**Branch:** `task-165-ec-distann-m3`. Measured recall effect of the M5 fold.

## Result (release, artifacts/fold-recall.log)

recall@10 vs exact truth on 2001 real rows, 50 queries: **A_full 0.9560 vs
B_fold 0.8900** (1801 built + 200 inserted+folded). The fold is correct (folded
rows found via graph traversal) but **costs ~0.066 recall@10** vs a full rebuild
with ~21% of the index folded.

## Interpretation

A "measure, don't assume" finding — folding is not recall-neutral. Attributed to
(1) append-if-free backlinks (full neighbors skip the back-edge) and (2)
head-sample-only candidate search (a fold batch does not interconnect). Both are
in the M5 follow-up set. Interim posture: REINDEX restores full recall; the
delta buffer + fold keep inserts correct + queryable meanwhile.

## Ask

Confirm the measurement method and the follow-up prioritization (full-reprune
backlinks + head-sample refresh to close the gap). Not closing the request.
