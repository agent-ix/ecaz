# SPIRE PQ-FastScan Deferral Diagnostics

## Checkpoint

- Code commit: `fef3ec03` (`Expose SPIRE payload scannability diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: operator-facing assignment payload scannability status for
  `ec_spire_index_options_snapshot(index_oid)`

## Summary

This checkpoint makes the SPIRE PQ-FastScan deferral explicit at the SQL
diagnostics surface:

- Added `assignment_payload_scannable`, `assignment_payload_status`, and
  `assignment_payload_recommendation` to
  `ec_spire_index_options_snapshot(index_oid)`.
- `turboquant` and `rabitq` report `assignment_payload_scannable = true` and
  `assignment_payload_status = 'supported'`.
- `pq_fastscan` reports `assignment_payload_scannable = false` and
  `assignment_payload_status = 'deferred_model_metadata'`, with recommendation
  text pointing at the missing grouped-PQ model metadata.
- Extended the existing SQL options snapshot coverage to assert both the
  supported RaBitQ case and an empty `pq_fastscan` index case. The empty-index
  case is intentional: it can expose the configured payload format without
  invoking row payload encoding or scan-time scorer binding.
- Updated the Task 30 plan to record this as a diagnostics-only hardening
  checkpoint.

This does not implement SPIRE PQ-FastScan scorer binding, persist grouped-PQ
model/codebook metadata for SPIRE assignment rows, change relation-backed
partition object storage, or alter `ec_ivf` PQ-FastScan behavior.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_options_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `210 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- This is not a measurement or recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- The status row documents the current SPIRE boundary: assignment payload
  formats backed by row-local scoring metadata are scannable now, while
  PQ-FastScan waits for durable grouped-PQ model metadata in the SPIRE storage
  design.
