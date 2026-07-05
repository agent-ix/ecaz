# Review Request: Task 41 SPIRE Custom Scan Planner Guards

Code commit: `943501a9276b8372c08fdd50e6e2cca0244d3bd8`

## Summary

This checkpoint wraps the direct PostgreSQL resources in
`src/am/ec_spire/custom_scan/planner.rs`.

- Reuses the existing `OpenIndexRelation` guard for planner candidate index
  eligibility checks.
- Adds small guards for placement SQL table relations, active snapshots,
  index scans, and tuple table slots.
- Replaces manual early-return cleanup in `custom_scan_index_has_sql_placement`
  with drop-owned cleanup.

## Safety Delta

- Baseline entries: `4321` -> `4319`.
- `src/am/ec_spire/custom_scan/planner.rs`: `39` -> `37`.
- The remaining planner entries are raw planner pointer/list manipulation and
  CustomPath/CustomScan construction, not paired relation/snapshot/scan/slot
  cleanup.

## Reviewer Focus

- Confirm `OpenIndexRelation` is acceptable for planner candidate eligibility
  even though the type is defined later in the included custom-scan module.
- Confirm `custom_scan_index_has_sql_placement` keeps drop order correct:
  slot, index scan, active snapshot, placement index, placement table.
- Confirm the early-return behavior remains fail-closed when any PostgreSQL
  resource open/register step returns null.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
