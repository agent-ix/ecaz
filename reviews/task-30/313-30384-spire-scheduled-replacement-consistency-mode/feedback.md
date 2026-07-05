# 30384 SPIRE Scheduled Replacement Consistency Mode — feedback

## What landed

Adds `input.consistency_mode != snapshot.epoch_manifest.consistency_mode`
rejection to `build_scheduled_replacement_epoch_draft_from_object_placements`
(line 1111-1116). Prevents publication from silently switching
consistency modes mid-replacement.

## Correctness

- This was a real hole. Before this guard, a scheduler that loaded the
  active manifest under one consistency mode could have built a draft
  with a different mode if the caller fabricated `input.consistency_mode`
  independently. Now the manifest's mode is the source of truth and the
  caller-supplied mode must agree.
- The downstream
  `validate_scheduled_replacement_execution_publish_plan_parts` (1411)
  also enforces `publish_plan.consistency_mode == input.consistency_mode`
  (via the relation/local validators in 30386/30388), and 30383 derives
  `publish_plan.consistency_mode` from the manifest. So consistency-mode
  preservation is now enforced at three independent seams.

## Status

Lands cleanly. Important guardrail; right place to add it.
