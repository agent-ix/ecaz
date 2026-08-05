---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 207 re-review corrections

This packet supersedes the Task 207 packet-005 wording called out by the
reviewer.

- Physical head construction now persists `head_construction` in the active
  head-state row and exposes it through `ec_distann_active_head_construction`.
  The marker is tied to the active candidate rather than inferred from an
  indirect digest change.
- The release lane's seed-control GUCs are compiled out. For the reviewed
  BW128 lane, the effective production derivation is therefore 256 seeds and
  width 256; the old 128/128 wording is withdrawn.
- The owner rows are withdrawn from membership/overlap evidence: that lane
  was head-independent and captured top-k 32 rather than the main top-k 200.
  No membership or overlap claim is made from it.
- The persisted-head/Vamana search path remains diagnostic. The shipped
  state is `training_landmarks_exact` when an explicit training relation is
  selected; this packet makes no default change and no production promotion.

The code checkpoint is `366a7973d`; the implementation and packet evidence
are pushed on the task branch. A future membership/overlap measurement still
needs head-sample IDs and matching top-k 200 predictions; those IDs were not
captured by the withdrawn owner run and are not fabricated here.
