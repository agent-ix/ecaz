# Review Request: IVF planner cost relation guard

Code commit: `1e878c0271a05d4f9b2d84994a6ee738a66afb90`

## Summary

This packet hardens the IVF planner cost callback in
`src/am/ec_ivf/cost.rs`.

- Added `OpenedCostIndexRelation`, an RAII guard that owns the
  `index_open` / `index_close` pair for the planner callback.
- Added explicit null checks for the callback arguments and `IndexPath`
  `indexinfo` before raw pointer dereferences.
- Kept the remaining raw relation reads narrow and documented with
  specific `// SAFETY:` invariants.
- Updated `scripts/unsafe_comment_baseline.txt` after the structural
  reduction.

## Baseline

- Before: 4733 entries.
- After: 4725 entries.
- Net change: 8 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm the RAII guard owns exactly the relation opened by the IVF planner
  callback and closes it on the normal callback path.
- Confirm the new null checks cover every callback pointer before use.
- Confirm the remaining `unsafe` sites are actual PG relation / planner state
  boundaries rather than wrapper candidates within this slice.

## Validation

- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-909.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
