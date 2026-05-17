# 30386 SPIRE Scheduled Relation Publish Plan Validation — feedback

## What landed

`validate_relation_scheduled_replacement_execution_publish_plan` is now
required at `publish_relation_scheduled_replacement_epoch` entry (line
1568). Checks epoch, consistency_mode, next_local_vec_seq against the
publish plan, then defers shape checks to the shared
`validate_scheduled_replacement_execution_publish_plan_parts`.

## Correctness

- The relation publisher now refuses to write objects if the execution
  input has drifted from the publish plan after the lock was taken — this
  is the seam that catches "caller mutated the input between plan and
  publish" bugs.
- Same shared `_parts` validator is reused, so local + relation
  validation is symmetric (verified again by 30388 on the local side).

## Status

Lands cleanly.
