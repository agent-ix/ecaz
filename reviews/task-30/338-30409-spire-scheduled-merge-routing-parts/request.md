# Review Request: SPIRE Scheduled Merge Routing Parts

## Summary

Task 30 SPIRE Phase 2 now has a pure merge routing preparation helper that
composes centroid recomputation, replacement-child construction, and parent
routing rewrite.

Changes:
- Add `SpireScheduledReplacementRoutingParts`.
- Add `build_scheduled_merge_replacement_routing_parts`.
- Return the rewritten parent plus replacement children for a checked merge
  decision and PID plan.
- Cover successful parent rewrite plus reused-PID and object-version
  rejections.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_merge_replacement_routing_parts --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

This advances merge scheduler invocation wiring while remaining pure and
local-testable. No measurement claims; no PG callback coverage.
