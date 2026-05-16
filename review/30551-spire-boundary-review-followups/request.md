# Review Request: SPIRE Boundary Review Follow-Ups

- Code commits:
  - `6feba1da` (`Add SPIRE top graph object codec`)
  - `5e8a2e8c` (`Share SPIRE centroid IP routing`)
- Feedback commits/files:
  - `d200e97d` (`Add SPIRE phase 5 boundary-replication review feedback`)
  - `review/30545-spire-recursive-boundary-replica-build/feedback/2026-05-06-01-reviewer.md`
  - `review/30546-spire-split-replacement-boundary-fanout/feedback/2026-05-06-01-reviewer.md`
  - `review/30547-spire-boundary-storage-accounting/feedback/2026-05-06-01-reviewer.md`
  - `review/30548-spire-boundary-recall-study/feedback/2026-05-06-01-reviewer.md`
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5 feedback follow-ups
- Agent: coder1

## Summary

This checkpoint addresses the actionable Phase 5 boundary-replication feedback:

- recursive builds now route primary placement through the same boundary route
  map for `boundary_replica_count = 0` and `> 0`, so turning replicas on does
  not silently switch the primary-placement metric;
- split replacement boundary routing now validates source vectors and
  centroids for non-empty, finite, non-zero vectors before scoring;
- leaf snapshot accounting now reads V2 leaves first, falls back to V1, checks
  row-counted assignments against the leaf header assignment count, and checks
  primary + boundary-replica role counts against the same header total.
- build route maps, scan routing objects, and split replacement routing now
  share one `rank_centroid_routes_by_ip` helper for inner-product scoring and
  tie-break ordering;
- `plan/design/spire-boundary-replication.md` now records why the local
  recall/storage evidence keeps boundary replicas default-off until Phase 7
  remote availability/read-throughput experiments.

The same code commit also includes the Phase 6 top-graph object codec; review
for that portion is tracked in `review/30550-spire-top-graph-build-draft/`.

## Files

- `src/am/ec_spire/build/recursive.rs`
- `src/am/ec_spire/build/routing_plan.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/scan/routing.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/update/materialization.rs`
- `src/am/ec_spire/update.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `plan/design/spire-boundary-replication.md`

## Review Focus

1. Confirm recursive default placement now uses the same inner-product route
   metric as boundary fanout while still allocating one `vec_id` per source row.
2. Check that split replacement validation is aligned with other route-vector
   validation surfaces.
3. Review whether the leaf snapshot row/header and role-count checks are strict
   enough for current writers without rejecting legitimate rows.
4. Confirm the shared centroid ranking helper preserves the existing ordering:
   higher inner product, lower centroid ordinal, lower child PID.
5. Check whether the design note correctly frames local recall evidence versus
   Phase 7 remote availability/read-throughput value.

## Validation

- `cargo test --lib recursive_build_coordinator_fans_out_boundary_leaf_rows --no-default-features --features pg18`
- `cargo test --lib split_replacement_materialization --no-default-features --features pg18`
- `cargo test --lib leaf_snapshot --no-default-features --features pg18`
- `cargo test --lib route_root_object_to_leaf_pids --no-default-features --features pg18`
- `cargo test --lib top_graph --no-default-features --features pg18`
- `git diff --check`
