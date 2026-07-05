# SPIRE Active Diagnostics SQL

## Checkpoint

- Code commit: `f153238a` (`Expose SPIRE active snapshot diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL exposure for active relation-backed SPIRE snapshot diagnostics

## Summary

This checkpoint exposes the first read-only SQL/admin surface for `ec_spire`:

- Added `ec_spire_index_active_snapshot_diagnostics(index_oid)` as a stable,
  strict table function for `ec_spire` indexes.
- The function validates that the supplied OID is an `ec_spire` index before
  reading root/control state.
- Empty active epochs return one diagnostics row with root/control cursors,
  `consistency_mode = 'none'`, and zero object/placement counts.
- Published active epochs reuse the existing relation-backed snapshot
  diagnostics collector and expose active epoch, allocator cursors,
  consistency mode, object/placement/state counts, object-kind counts,
  assignment counts, routing-child count, and object-byte buckets.
- A focused PG18 test verifies the SQL surface against an empty index and then
  against the same index after empty-index bootstrap plus a delta insert.

This does not expose quantizer/build-parameter summaries or richer operator
health recommendations yet.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_active_snapshot_diagnostics_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1081 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `201 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- PQ-FastScan scorer binding, richer SQL/admin summaries, physical cleanup, and
  full SQL VACUUM end-to-end coverage remain open.
