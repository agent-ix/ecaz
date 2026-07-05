# Review Request: SPIRE Boundary Storage Accounting

- Code commit: `c3497c76` (`Expose SPIRE boundary replica storage counters`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Agent: coder1

## Summary

This checkpoint makes boundary replica physical storage growth visible in leaf
snapshot diagnostics.

`ec_spire_index_leaf_snapshot` now separates assignment counts into:

- base primary assignments;
- base boundary replica assignments;
- delta insert boundary replica assignments;
- effective boundary replica assignments.

The base counts are read from the leaf object assignment payload, with v1/v2
leaf object fallback matching the existing snapshot reader path. Delta insert
counts now separately include rows flagged as both `BOUNDARY_REPLICA` and
`DELTA_INSERT`.

The existing boundary-replica PG18 fixture now asserts both build-time replica
storage and post-build insert replica storage, so the diagnostic proves physical
assignment growth separately from logical deduped scan output.

## Review Focus

1. Confirm the new SQL diagnostic columns are placed coherently for named and
   positional callers of `ec_spire_index_leaf_snapshot`.
2. Check that the v1/v2 leaf object fallback used for counting base assignment
   roles matches the snapshot row collection semantics.
3. Verify delta boundary insert accounting should count only replica insert
   rows and not delete deltas.
4. Confirm `effective_boundary_replica_assignment_count = base boundary +
   delta boundary insert` is the intended current definition while boundary
   delete deltas remain invalid.

## Validation

- `cargo test --lib leaf_snapshot --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_build_writes_and_dedupes_scan`
- `git diff --check`

PG17 was not run; this slice is centered on the PG18 SPIRE Phase 5 diagnostic
surface.
