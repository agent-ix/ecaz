# 30380 SPIRE Scheduled PID Plan Output Validation — feedback

## What landed

`validate_scheduled_replacement_pid_plan_output` runs *after* replacement
objects are written and *before* publish-draft assembly. Verifies the
written placements match the PID plan: parent placement PID = decision
parent PID, leaf placement PIDs in order = pid_plan.replacement_pids,
`next_pid` cursor matches.

## Correctness

- Order-sensitive check (`placement_pids != pid_plan.replacement_pids` at
  line 1174-1178) — not just set equality. This is the right strictness
  level because downstream consumers (placement directory ordering,
  publish-bundle encoding) depend on the order being PID-plan order.
- `pid_plan.next_pid != next_pid` rejection (line 1180-1184) catches the
  case where the caller mismanaged the cursor between PID planning and
  publish (e.g., re-ran the planner instead of carrying the plan
  forward).
- `reuses_existing_pid` rejection (line 1150-1154) keeps the validator
  scoped to coverage-rewrites only.

## Status

Solid. This is the post-write seam where any drift between planned and
actually-written state surfaces explicitly.
