---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 03
---

# Task 207 packet 006 — re-review corrections: ACCEPT the corrections; one substantive question remains open for disposition

Verified against the code and artifacts:

- **Activation marker is real end-to-end.** `head_construction` is persisted
  in `ec_distann_generation_head_state` (schema + upgrade SQL with proper
  default-drop), written from build options in `build_epoch`, surfaced by
  the new `ec_distann_active_head_construction` function (SECURITY DEFINER +
  search_path pinned + PUBLIC revoked, consistent with the sibling
  functions), attested into the fixture's `physical_benchmark_head_policy`
  line, and asserted by the lifecycle test. The marker claim in FR-080 is
  now true for the physical path.
- **The withdrawals are the right calls, cleanly executed**: the
  head-independent owner-oracle table is withdrawn from membership/overlap
  evidence with the reason stated; the effective-seed record is corrected
  (BW128 ⇒ 256/256 on the release build); packet 005 carries supersede
  notes pointing here. Declining to reconstruct membership numbers from
  artifacts that never captured head-sample IDs is correct — that is the
  no-fabrication rule applied properly.
- **Phase-2 disposition is now recorded**: production stays
  `training_landmarks_exact` when explicitly selected; the persisted-head
  Vamana path remains a diagnostic arm; no silent promotion.
- **LRU eviction fixed** with a focused regression test
  (`cache_eviction_removes_oldest_matching_index`) that covers the exact
  inverted case, keeps two entries per index, and preserves unrelated
  indexes.

Remaining items:

1. **P2 — the pre-registered hypothesis is still untested, and the packet
   should say what happens to it.** The construction decision (keep
   `stitched_bfs`; union not promoted) is closed on end-to-end evidence and
   stands. But Task 207's motivating question — is head *membership* the
   recall bound, and what does the membership picture look like under each
   construction — now has no measurement anywhere in the bucket, and the
   packet stops at "IDs were not captured." The capture is cheap now: the
   head-sample IDs are in `ec_distann_generation_head_state`/sample chain on
   any built fixture, and one 10k bring-up per arm plus the existing
   ground-truth predictions yields membership and overlap@k offline. Either
   run that one diagnostic, or record an explicit open-question disposition
   naming where the membership work lands (Task 185's selection-objective
   lane and/or Task 210's sharding lane are the natural owners — operator's
   pick, not mine to file). Task 207 should not close with the hypothesis
   silently dropped between packets.
2. **P3 — the lifecycle test asserts the marker only for the stitched
   default.** Add the `partition_union` assertion (build with the reloption,
   expect `partition_union`/`true`) so the marker distinguishes arms in a
   test, not only in fixture logs.
3. **P3 — carried from the 206 bucket:** the structured `scan_round` capture
   into `results.jsonl` still doesn't fire; one shared fix covers both
   lanes.

With item 1 dispositioned (either the small measurement or the recorded
handoff), Task 207 is closeout-ready from the review side: construction A/B
deconfounded and decided, search-path status recorded, marker attested,
spec and packet record consistent with what actually ran.
