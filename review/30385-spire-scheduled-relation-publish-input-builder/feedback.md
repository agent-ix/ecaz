# 30385 SPIRE Scheduled Relation Publish Input Builder — feedback

## What landed

`build_relation_scheduled_replacement_execution_input_from_publish_plan`
is the pure bridge from a checked publish plan + decision + PID plan +
caller-supplied parts → `SpireRelationScheduledReplacementExecutionInput`.
Preserves the planned epoch, active consistency mode, and local vec_seq
cursor from the publish plan.

## Correctness

- The builder calls
  `validate_scheduled_replacement_execution_publish_plan_parts` (the
  shared validator) and only constructs the input on success. Drift
  cannot escape this seam.
- `SpireRelationScheduledReplacementExecutionParts` deliberately omits
  `placement_write_evidence` (the relation publisher generates it from
  real placement-row writes); the local analogue in 30387 includes it
  because local dry-run tests fabricate the evidence. Right asymmetry.

## Status

Lands cleanly. Pure helper, well-scoped.
