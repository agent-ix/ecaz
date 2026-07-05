# Review Request: DiskANN Planner Cost Relation Guard

## Summary

This slice hardens the DiskANN planner cost callback by validating PostgreSQL
callback pointers and replacing manual index close handling with an owning
guard.

Code checkpoint: `97810799637a182a514c1c73592e9a2f7ac5960b`

## Safety Handling

- Added `OpenedCostIndexRelation`, which owns the `index_open` / `index_close`
  pair for planner cost estimation.
- Added fail-closed null checks for:
  - `path`
  - cost/selectivity/correlation/page output pointers
  - `path.indexinfo`
- Kept `compute_amcostestimate()` unsafe because it still consumes a raw
  PostgreSQL `Relation` pointer and reads relation/options/metadata through
  PostgreSQL APIs.
- Added SAFETY comments to the remaining pointer/API boundaries in
  `compute_amcostestimate()`.

This makes the callback more robust while keeping the unavoidable relation
pointer contract explicit.

## Baseline Delta

- Before: 4,738 unsafe baseline entries across 106 files.
- After: 4,733 unsafe baseline entries across 106 files.
- Net: 5 entries removed.

`src/am/ec_diskann/cost.rs` moved from 6 baseline entries to 1.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `git diff --check HEAD^ HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passes with the existing PostgreSQL header warnings and existing
unused SPIRE re-export warning.

## Artifacts

- `artifacts/unsafe-baseline-before.log`
- `artifacts/unsafe-baseline-after.log`
- `artifacts/audit-unsafe.log`
- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18.log`

## Review Focus

- Does `OpenedCostIndexRelation` correctly represent planner-owned index
  relation lifetime for this callback?
- Are the null checks strict enough for PostgreSQL planner callback inputs?
- Should `compute_amcostestimate()` remain unsafe, or is there a better
  relation wrapper boundary for DiskANN diagnostics/cost code?
