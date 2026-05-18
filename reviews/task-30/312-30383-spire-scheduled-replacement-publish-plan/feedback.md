# 30383 SPIRE Scheduled Replacement Publish Plan — feedback

## What landed

`plan_scheduled_replacement_publish_epoch` produces the under-publish-lock
plan: binds root/control active epoch, allocator cursors, the active epoch
manifest, the checked decision, and the fresh PID plan into a
`SpireScheduledReplacementPublishPlan { epoch, consistency_mode, next_pid,
next_local_vec_seq }`.

## Correctness

- Active-epoch agreement check is **three-way**: root/control,
  active-epoch manifest, and decision must all carry the same epoch (lines
  1196-1206). Catches publish-lock loss, manifest swap, or decision
  staleness.
- Manifest must be `SpireEpochState::Published` (line 1208-1213) — a
  retired or draft manifest cannot be the predecessor of a publish.
- PID-plan integrity is checked thoroughly: fresh PIDs, count vs decision,
  no duplicates, no PID below `root_control.next_pid`, every replacement
  PID strictly less than `pid_plan.next_pid`, plan's `next_pid` not behind
  root/control. This is the canonical seam where the allocator-cursor
  contract is enforced post-PID-allocation.
- Resulting plan carries `consistency_mode` from the *manifest*, which is
  later cross-checked against snapshot in 30384.

## Status

Solid. Heavy validation but appropriate — this is the gate before any
mutation.
