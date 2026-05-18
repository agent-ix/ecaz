# Review Request: SPIRE Recursive Boundary Replica Build

- Code commit: `1aed6c7a` (`Fan out SPIRE recursive boundary builds`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Agent: coder1

## Summary

This checkpoint extends Phase 5 boundary replica assignment fanout into
recursive SPIRE builds.

The recursive build coordinator now carries source vectors alongside assignment
rows and receives the parsed `boundary_replica_count`. After allocating leaf
PIDs and building the recursive routing draft, it builds a leaf route map from
the trained centroids and selected leaf PIDs. When `boundary_replica_count > 0`,
each source vector is routed through the same top-N boundary predicate used by
single-level builds. The coordinator writes one primary row and bounded
`BOUNDARY_REPLICA` rows with the same `vec_id` into the selected recursive leaf
objects.

The default recursive path remains primary-only when
`boundary_replica_count = 0`.

## Review Focus

1. Confirm recursive build fanout uses the same leaf-level top-N predicate as
   single-level boundary replication.
2. Check that the coordinator still advances `next_local_vec_seq` once per
   source vector, not once per physical replica row.
3. Verify preserving empty leaf inputs for routed-but-empty leaves remains
   compatible with recursive placement validation and diagnostics.
4. Confirm the PG18 regression's scan assertion is framed correctly: recursive
   routing currently returns the selected upper-level subtree, so the test
   proves physical fanout plus dedupe for routed rows rather than full recall.

## Validation

- `cargo test --lib recursive_build_coordinator_fans_out_boundary_leaf_rows --no-default-features --features pg18`
- `cargo test --lib recursive_build_coordinator_assembles_epoch_input_from_centroid_plan --no-default-features --features pg18`
- `cargo test --lib recursive_build_coordinator --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_recursive_boundary_replica_build_dedupes`
- `git diff --check`

PG17 was not run; this slice is centered on the PG18 SPIRE Phase 5 recursive
build path.

## Notes

Split/merge replacement fanout remains open for a later Phase 5 checkpoint.
