# SPIRE PQ-FastScan Build Deferral Coverage

## Checkpoint

- Code commit: `0c82fc80`
  (`Cover SPIRE pq fastscan deferral error`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: PG18 error-path coverage for populated `pq_fastscan` SPIRE builds

## Summary

This checkpoint adds focused coverage for the current SPIRE PQ-FastScan
deferral boundary.

- Empty `ec_spire` indexes with `storage_format = 'pq_fastscan'` can expose
  the configured option through diagnostics.
- Populated builds must fail until SPIRE persists grouped-PQ model metadata.
- The new PG18 test creates a populated `ec_spire` index with
  `storage_format = 'pq_fastscan'` and asserts the explicit deferral error:
  `ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model`.
- The Task 30 plan now records this error-path coverage.

This does not implement SPIRE PQ-FastScan model persistence or scan scoring.

## Changed Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_pq_fastscan_populated_build_reports_deferral --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1114 filtered out`
  - surfaced error:
    `ec_spire ambuild found invalid indexed ecvector: ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `234 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check` before commit

## Notes

- PQ-FastScan remains deferred for SPIRE until grouped-PQ model/codebook
  metadata is persisted in the SPIRE storage model and loaded by the scan
  scorer.
- `ec_ivf` PQ-FastScan behavior is unchanged.
