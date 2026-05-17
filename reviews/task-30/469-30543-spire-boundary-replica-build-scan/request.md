# Review Request: SPIRE Boundary Replica Build and Scan

- Branch: `main`
- Code commit: `d9f89caa` (`Write SPIRE boundary replicas in populated builds`)
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Scope: first runtime fanout slice for populated single-level relation-backed
  builds

## Summary

This checkpoint turns the Phase 5 planning surface into executable
single-level build behavior.

It:

- adds assignment helpers that allocate one `vec_id` per source vector and
  emit one `PRIMARY` row plus bounded `BOUNDARY_REPLICA` rows;
- wires populated single-level relation-backed builds to write replica rows
  when `boundary_replica_count > 0`;
- keeps the same top-N leaf route ordering from the design checkpoint;
- adds a scored-visible assignment predicate so scans score primary and
  boundary-replica rows, while update/vacuum helpers still use the
  primary-only predicate where identity ownership matters;
- keeps existing `VecIdDedupeEnabled` candidate merging as the final scan
  duplicate-control boundary;
- adds PG18 coverage proving a three-row build with
  `boundary_replica_count = 1` writes six leaf assignments and still returns
  three deduped heap rows.

This does not yet fan out insert-delta writes, recursive builds, or split/merge
replacement leaves. Those remain tracked in the task file.

## Files

- `src/am/ec_spire/assign.rs`
- `src/am/ec_spire/build/drafts.rs`
- `src/am/ec_spire/storage/helpers.rs`
- `src/am/ec_spire/scan/candidates.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Confirm the build helper correctly reuses one `vec_id` across primary and
   replica assignment rows.
2. Confirm scan visibility should include `BOUNDARY_REPLICA` rows only at the
   scored-candidate boundary, while update/vacuum keep primary-only semantics.
3. Confirm the relation-backed populated single-level scope is acceptable for
   the first fanout slice.
4. Check whether the task tracker wording is clear about remaining insert,
   recursive, and replacement fanout gaps.

## Validation

- `cargo test --lib boundary --no-default-features --features pg18`
- `cargo test --lib rank_routed_leaf_rows_by_ip_keeps_best_visible_vec_id_candidate --no-default-features --features pg18`
- `cargo test --lib assignment_visibility_helpers_match_primary_and_delta_semantics --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_boundary_replica_build_writes_and_dedupes_scan`
- `git diff --check`
