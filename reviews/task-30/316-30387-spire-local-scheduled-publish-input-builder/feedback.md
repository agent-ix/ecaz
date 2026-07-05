# 30387 SPIRE Local Scheduled Publish Input Builder — feedback

## What landed

`build_local_scheduled_replacement_execution_input_from_publish_plan` —
the local analogue of 30385. `SpireLocalScheduledReplacementExecutionParts`
includes `placement_write_evidence` (local dry-run tests fabricate it).

## Correctness

- Same shared `validate_scheduled_replacement_execution_publish_plan_parts`
  call as the relation builder, so PID-cursor / replacement-child order /
  leaf-input validation is identical.
- Output preserves `publish_plan.epoch`, `publish_plan.consistency_mode`,
  and `publish_plan.next_local_vec_seq` — the three fields the publish
  plan owns authoritatively.

## Status

Lands cleanly. Local + relation builders are now symmetric, as they
should be.
