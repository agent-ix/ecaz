# Review Request: SPIRE Relation Scheduled Split Execution Parts

## Summary

Task 30 SPIRE Phase 2 now has pure split-side composition for scheduled
relation execution parts.

Changes:
- Add `build_scheduled_split_replacement_routing_parts`.
- Add `build_relation_scheduled_split_replacement_execution_parts`.
- Bind caller-trained split centroids to replacement PIDs and rewrite the
  parent routing object.
- Order routed split leaf inputs by the PID plan before relation execution.
- Cover routing success, centroid-count/decision-mode rejection, split parts
  composition, and leaf-input drift.
- Update the Phase 2 checklist.

## Validation

- `cargo test scheduled_split_replacement_routing_parts --lib`
- `cargo test relation_scheduled_split_replacement_execution_parts --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

Actual split centroid training remains a live scheduler responsibility. This
checkpoint provides the checked composition seam once centroids and routed leaf
inputs are available.
No measurement claims; no PG callback coverage.
