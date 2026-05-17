# SPIRE Empty PQ-FastScan Scan Coverage

## Checkpoint

- Code commit: `bac8d10e`
  (`Cover SPIRE empty pq fastscan scan`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: PG18 coverage for empty `pq_fastscan` SPIRE scans

## Summary

This checkpoint covers the safe empty-index side of SPIRE's current
PQ-FastScan deferral.

- Added a PG18 test that builds an empty `ec_spire` index with
  `storage_format = 'pq_fastscan'`.
- The test forces index usage and confirms an ordered scan returns zero rows.
- The plan now distinguishes empty `pq_fastscan` indexes, which have no
  assignments to score, from populated `pq_fastscan` builds, which remain
  blocked until SPIRE persists grouped-PQ model metadata.

This does not implement populated PQ-FastScan build or scan scoring for SPIRE.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_empty_pq_fastscan_build_scan_no_rows --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1115 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `235 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check` before commit

## Notes

- Populated SPIRE PQ-FastScan build deferral coverage is in packet `30327`.
- `ec_ivf` PQ-FastScan behavior is unchanged.
