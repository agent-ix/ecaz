# Review Request: SPIRE Scheduled Parent Routing Loader

## Summary

Task 30 SPIRE Phase 2 now has a checked helper for loading the
decision-bound parent routing object from the active snapshot before scheduled
replacement execution composes routing children or rewrites the parent.

Changes:
- Validate decision shape and active snapshot epoch.
- Require an available parent placement.
- Read the parent through `SpireObjectReader::read_routing_object`.
- Recheck loaded parent PID and affected-leaf child coverage.
- Add local object-store coverage for success, stale epoch, wrong object kind,
  and missing affected child.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_replacement_parent_loader --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This advances live scheduler invocation wiring without touching PostgreSQL
callbacks. No measurement claims; no PG callback coverage.
