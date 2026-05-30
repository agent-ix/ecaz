# Review Request: Task 68 Packet 004 Zero-Replica Leaf Row Fast Path

Code commit: `c8f98a71da07e8d1417642fcbbe558ce0ae942d9`

## Summary

This is the first Task 68 P0 implementation slice after packet 003 identified `draft_leaf_rows_ms` as the hot subphase.

The measured path used `boundary_replica_count=0`, but `build_recursive_leaf_rows_by_pid` still routed every source vector through `route_boundary_assignment_for_vector(..., boundary_replica_count + 1)`. That repeated nearest-route work across the route map even though replicas were discarded and the primary leaf PID was already known from `assignment_indexes`.

The change adds a zero-replica fast path:

- derive `primary_pid` directly from `assignment_indexes` and the existing route map
- build placements through the same `build_boundary_leaf_assignment_placements_with_identity` helper to preserve validation and local/global vec-id allocation behavior
- leave the boundary-replica path unchanged for `boundary_replica_count > 0`

## Phase-1 Backreference

Packet 003 measured:

```text
draft_leaf_rows_ms=19182
total_ms=22482
```

This slice targets that 85.3 % wall-time cap directly.

## Validation

Artifact manifest: `artifacts/manifest.md`

```text
cargo test -p ecaz --lib am::ec_spire::build --no-default-features --features pg18
```

Result:

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 1875 filtered out
```

## Measurement

The follow-up measurement packet should repeat the 10k and 100k `CREATE INDEX` split against the same fixture tables and show whether `draft_leaf_rows_ms` collapses as expected.
