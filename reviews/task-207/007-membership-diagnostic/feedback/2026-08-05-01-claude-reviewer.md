---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-05
seq: 01
---

# Task 207 packet 007 — membership diagnostic: ACCEPT; Task 207 review is closed

This is the measurement the task was missing, and I verified it
independently: recomputing from the committed `physical-head-membership.json`
and prediction artifacts reproduces the packet's numbers — head-set overlap
exactly 2004/4096 = 0.4893, membership@32 = 0.4319 (stitched) vs 0.4848
(union) to four decimals, membership@200 = 0.4520 vs 0.5411 against the
cited 0.4503/0.5389 (small denominator-handling difference for queries with
fewer than 200 rows; same conclusion — state the denominator convention if
this table is ever cited onward). Both membership files carry the persisted
construction marker and 4,096 IDs, so arm attribution is direct, not
inferred.

The finding is the important part, and it is well-stated: **partition-union
raises head membership materially (+5.3 pts @32, +8.9 pts @200) while
end-to-end recall does not move (0.9486 → 0.9468).** Together with the
deconfounded construction A/B, the pre-registered hypothesis is now
dispositioned by measurement on both halves: union does change head
composition (only 49% landmark overlap with stitched) and does improve
membership — and membership is *not* the binding constraint at this
operating point. That redirects future head work toward the selection
objective / search interaction (Task 185's lever) rather than pool
construction, which is exactly what a diagnostic packet should deliver.

Supporting items all verified: the single-owner digest mismatch fix in
`build_epoch` correctly unifies the membership-only condition across
digest/graph/persistence (a real bug, caught by actually running the pgrx
test — noted with approval); the lifecycle test now builds and publishes a
`partition_union` epoch and asserts the marker flips; the membership JSON is
registered as a step artifact and parsed as a structured metric with test
coverage.

Two notes, non-blocking:

- The membership metric is computed over *returned prediction IDs*, not
  oracle-selected seeds or ground-truth neighbors. The direction of the
  conclusion is robust to that choice (membership rose, recall didn't), but
  the packet should not be cited later as measuring oracle-seed coverage.
- The two ~40k-line prediction JSONs are at the upper end of what a packet
  should carry. They are legitimate source evidence for the offline
  computation and are allowed; keep this a one-time capture rather than a
  pattern.

All Task 207 review items are now resolved: construction A/B deconfounded
and decided (keep `stitched_bfs`), marker attested end-to-end with tests,
search-path disposition recorded, packet record corrected to what actually
ran, and the membership hypothesis measured rather than dropped. Task 207 is
done from the review side; closeout/merge is the operator's call.
