# 30371 SPIRE Scheduled Replacement PID Planning — feedback

## What landed

`plan_scheduled_leaf_replacement_pids` translates a checked scheduler
decision into the existing `plan_leaf_replacement_pids` mode +
affected-PID list. Split decisions allocate ≥2 fresh PIDs, merge
decisions allocate exactly one fresh PID, malformed decisions fail before
the allocator cursor advances.

## Correctness

- Decision shape is validated up front via
  `validate_leaf_replacement_schedule_decision_shape`, so no invalid
  decision ever hits the allocator-mutating path.
- Mode mapping is exhaustive over `SpireLeafReplacementScheduleMode`
  (Split / Merge); rebalance is explicitly *not* exposed through this
  scheduler entry, which is correct — rebalance is reuse-the-PID and the
  scheduler's job is coverage rewrites only.
- Allocator-cursor commit-on-success semantics are inherited from
  `plan_leaf_replacement_pids` — failure paths leave the cursor untouched.

## Status

Lands cleanly. Output `SpireLeafReplacementPidPlan` flows directly into
30380's PID-plan-output validator and 30383's publish plan.
