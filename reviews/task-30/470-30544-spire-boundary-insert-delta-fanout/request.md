# Review Request: SPIRE Boundary Insert Delta Fanout

- Code commit: `d13248da` (`Fan out SPIRE boundary insert deltas`)
- Branch: `main`
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Agent: coder1

## Summary

This checkpoint extends boundary replication from populated single-level builds
into post-build inserts.

For non-empty SPIRE indexes, `aminsert` now:

- routes the inserted source vector with `nprobe = boundary_replica_count + 1`;
- treats the first routed leaf as the primary placement and the remaining
  routed leaves as boundary replica placements;
- allocates one shared `vec_id` for the inserted source row;
- writes one insert-delta object per selected target leaf, each parented to the
  corresponding base leaf placement;
- marks the primary row as `PRIMARY | DELTA_INSERT` and replica rows as
  `BOUNDARY_REPLICA | DELTA_INSERT`.

Delta object validation now accepts insert deltas for either primary or boundary
replica rows, but still requires exactly one of those scored roles. Delete
deltas continue to reject boundary replica rows.

The existing boundary-replica PG18 fixture now also inserts a fourth row after
build, asserts the leaf snapshot sees two insert-delta assignments with
`boundary_replica_count=1`, and confirms ordered scan output remains deduped.

## Review Focus

1. Check that the non-empty `aminsert` path uses the active snapshot routing
   order correctly for primary plus bounded boundary replica placement.
2. Verify that one delta object per target leaf is the right shape for current
   placement-directory and local-store routing semantics.
3. Confirm delta assignment validation is neither too permissive nor too narrow
   for boundary replica insert deltas.
4. Confirm the regression demonstrates both physical fanout and logical scan
   dedupe for post-build inserts.

## Validation

- `cargo test --lib build_boundary_insert_delta_assignment_placements_sets_delta_flags --no-default-features --features pg18`
- `cargo test --lib delta_partition_object_rejects_invalid_delta_flags --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_build_writes_and_dedupes_scan`
- `git diff --check`

PG17 was not run; this slice touches the PG18-centered SPIRE Phase 5 boundary
replication path.

## Notes

This checkpoint does not implement boundary replica fanout for recursive builds
or split/merge replacement publication. Those remain tracked Phase 5 items.
