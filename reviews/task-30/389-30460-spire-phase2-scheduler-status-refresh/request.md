# Review Request: SPIRE Phase 2 Scheduler Status Refresh

## Summary

Task 30 SPIRE Phase 2 plan text now reflects that the live manual scheduler
entrypoint and selected split/merge publish paths have landed.

Changes:
- Replace stale "scheduler remains open" notes in earlier helper checklist
  entries with references to the later live scheduler wiring.
- Update split/merge trigger entries to say those candidates are now consumed by
  the manual scheduler.
- Leave still-open follow-ups unchanged: old-epoch cleanup, concurrency stress,
  hierarchy work, placement benchmarks, and distributed planning.

## Validation

- `git diff --check`

## Notes

Documentation/status-only checkpoint. No measurement claims.
