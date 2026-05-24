# Task 50 Review Request: SPIRE Cost Snapshot Safe Signatures

## Summary

This slice converts SPIRE cost diagnostic helpers from unsafe raw-relation entry points to safe `SpireLiveIndexRelation` helpers. The PostgreSQL AM callback functions remain the raw ABI boundaries.

Code commit: `cd6768e0250b3d2d392c653148ad4c4cfefa3776`

## What Changed

- Converted `index_cost_snapshot` and `index_cost_tuning_snapshot` to take `SpireLiveIndexRelation`.
- Converted internal cost helpers `compute_amcostestimate`, `spire_tree_height_callback_value`, `cost_active_snapshot_diagnostics`, and `cost_index_hierarchy_snapshot` to use the typed wrapper.
- Updated `ec_spire_amcostestimate` and `ec_spire_amgettreeheight` to construct `SpireLiveIndexRelation` at the AM callback boundary.
- Updated SQL wrappers in `src/lib.rs` to use `with_spire_live_index_relation!`.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds substantial residual unsafe under `src/` (`1960` line hits), so the Wave 5 closeout gate from packet 030 is not satisfied.

## Review Focus

- Please verify cost SQL diagnostics no longer expose safe-callable helpers through raw `pg_sys::Relation` unsafe signatures.
- Please check that planner AM callbacks remain the correct raw boundary for `ec_spire_amcostestimate` and `ec_spire_amgettreeheight`.
- Please check the new explicit `relation_main_fork_block_count` unsafe blocks are the right residual local boundaries.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed cost unsafe helper signatures, generic raw SQL wrapper use, and `checked_live_index_relation`
- `make UNSAFE_LEDGER=reviews/task-50/320-spire-cost-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/320-spire-cost-snapshot-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/320-spire-cost-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1960` (down from packet 319 `1973`)
- Unsafe ledger rows: `1368`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-cost-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
