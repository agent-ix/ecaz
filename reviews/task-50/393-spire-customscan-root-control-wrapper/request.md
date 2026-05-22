# Review Request: SPIRE CustomScan Root-Control Wrapper

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/393-spire-customscan-root-control-wrapper`
- code commit: `441ee67541de99f5570c8ae182a772ec1a5abeb0`
- scope:
  - `src/am/ec_spire/custom_scan/planner.rs`
  - `src/am/ec_spire/coordinator/snapshots.rs`

## Summary

This slice removes the direct root/control page unsafe from SPIRE CustomScan
planner eligibility. `custom_scan_index_eligibility_result` already receives a
`SpireLiveIndexRelation`, so it now uses the existing typed
`SpireLiveIndexRelation::root_control()` boundary instead of calling
`page::read_root_control_page` directly.

The root-control method visibility is narrowed to `pub(in crate::am::ec_spire)`
so sibling SPIRE modules can use the typed relation wrapper without making the
private root-control type crate-public.

## Unsafe Count

- before this slice: `1124`
- after this slice: `1123`
- removed caller-side unsafe: `src/am/ec_spire/custom_scan/planner.rs`

## Plan Coverage

- Program: P2, PostgreSQL Handle Views
- Wave/tranche: Wave 2 SPIRE production fanout, CustomScan planner relation
  view reuse.
- Disposition: caller unsafe was absorbed into the existing
  `SpireLiveIndexRelation` root/control boundary. The residual page decode
  unsafe remains in the owner boundary that already validates and decodes the
  root/control page.

## Validation

Artifacts are packet-local under `reviews/task-50/393-spire-customscan-root-control-wrapper/artifacts/`.

- `rustfmt-check.log`: passed; only existing stable-toolchain warnings for
  unstable rustfmt settings.
- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML
  re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: no matches.
- `src-unsafe-count.log`: `1123`.
- `unsafe-ledger-generate.log`: generated `1123` current `src` ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers `1123` current unsafe rows.
