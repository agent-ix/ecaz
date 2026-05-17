# Review Request: SPIRE Scheduled PID Plan Output Validation

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: 30 SPIRE Phase 2 update mechanics
- Scope: pure validation between scheduled PID allocation and publish-draft
  assembly.

## Summary

- Added `validate_scheduled_replacement_pid_plan_output`.
- The helper validates scheduler decision shape, requires fresh replacement
  PIDs, checks PID-plan count against the decision, checks the replacement
  parent placement PID, checks replacement leaf placement PIDs in PID-plan
  order, and verifies the final `next_pid` cursor matches the PID plan.
- Added focused tests for matching outputs plus leaf order, parent placement,
  and cursor mismatch rejection.
- Updated the Task 30 Phase 2 checklist to record this scheduler guardrail.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test scheduled_replacement_pid_plan_output --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

- This is intended to run after replacement objects are written and before
  replacement epoch draft assembly.
- No measurement claims are made in this packet.
