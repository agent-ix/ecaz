# SPIRE Options Diagnostics SQL

## Checkpoint

- Code commit: `c125fa6f` (`Expose SPIRE options diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL exposure for `ec_spire` relation options and session scan
  overrides

## Summary

This checkpoint extends the read-only SQL/admin surface for `ec_spire`:

- Added `ec_spire_index_options_snapshot(index_oid)` as a stable, strict table
  function for `ec_spire` indexes.
- The function validates that the supplied OID is an `ec_spire` index.
- The snapshot reports relation `nlists`, `nprobe`, `rerank_width`,
  `training_sample_rows`, `seed`, `pq_group_size`, `storage_format`, and the
  resolved assignment payload format.
- It also reports session overrides for `ec_spire.nprobe` and
  `ec_spire.rerank_width` when set.
- A focused PG18 test verifies reloption reporting, `storage_format` to payload
  format resolution, and session override reporting.

This does not add health recommendations, recall/latency measurements, or
PQ-FastScan grouped-PQ model diagnostics.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_options_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `202 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- PQ-FastScan scorer binding, physical cleanup/compaction, health
  recommendations, and full SQL VACUUM end-to-end coverage remain open.
