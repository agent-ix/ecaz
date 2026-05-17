# 30379 SPIRE Scheduled Replacement Successor Epoch — feedback

## What landed

Tightens `build_scheduled_replacement_epoch_draft_from_object_placements`
to require `input.epoch == decision.active_epoch + 1` (1101-1110). Adds
focused rejection coverage for skipped epochs.

## Correctness

- The immediate-successor invariant is now enforced in three places:
  the publish-draft helper (this packet), the object writer
  (30378's `write_scheduled_replacement_objects_with_writer`), and the
  publish plan (30383's `plan_scheduled_replacement_publish_epoch`).
  All three derive `expected_epoch` via `checked_add(1)` against
  `decision.active_epoch`, so epoch overflow is rejected consistently.
- Matches the existing insert/vacuum replacement publish contract — no
  divergence in scheduler behavior.

## Status

Lands cleanly. Trivial guardrail, but it closes a real publish-state
hole: a scheduler that planned for epoch N+1 cannot accidentally publish
N+2.
