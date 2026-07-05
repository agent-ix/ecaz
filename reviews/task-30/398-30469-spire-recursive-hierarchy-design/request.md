# Review Request: SPIRE Recursive Hierarchy Design

## Summary

Task 30 Phase 3 now has a durable recursion design checkpoint in
`plan/design/spire-recursive-hierarchy.md`.

The design records:

- Leaf-distance level numbering: leaves at level 0, root at the maximum active
  hierarchy level, and the current single-level shape as root level 1 over leaf
  level 0.
- Root/internal/leaf PID invariants for strict active epochs, including child
  kind/level compatibility and unique child PIDs within one routing parent.
- The first recursive routing reference format using the existing flat routing
  object body: child PIDs, centroid ordinals, and centroid blocks.
- Per-level build and routing metadata expectations, including default
  `nprobe` resolution while the user-facing reloption surface remains
  single-value.
- A bottom-up recursive build coordinator that consumes level-1 leaf centroids
  as the next level's training input and keeps single-level build as the
  degenerate case.
- Centroid materialization boundaries for diagnostics/rebuild/update planning.
- A level-local routing primitive and the multi-level scan descent contract.
- Explicit deferrals for boundary replication, top-level graph routing,
  multi-store/remote placement, background scheduling, old-epoch reclamation,
  recursive update propagation, product-scale measurements, and PG17
  validation.

## Review Focus

Please review whether the design is tight enough for the next implementation
slices:

- hierarchy metadata/diagnostics for recursive-capable root/internal/leaf
  objects
- pure recursive build draft helpers in memory
- centroid materialization/read helpers
- level-local routing primitive tests

In particular, check the level-numbering contract and whether root/internal
parent/child PID invariants are sufficient to fail closed on malformed strict
epochs before relation-backed recursive scan code descends.

## Validation

- `git diff --check`

## Notes

Documentation/design-only checkpoint. No measurement claims.
