# Task 50 Review Request: IVF Admin Wrapper Boundaries

## Summary

This slice removes the unsafe API requirement from IVF admin diagnostic wrappers where the open relation lifetime is already enforced by the SQL guard layer.

Code commit: `48dee69cd2e3a320f4d5c7bd148a74a5bb808735`

## Changes

- Converted `ec_ivf::index_drift_snapshot`, `ec_ivf::index_admin_snapshot`, `ec_ivf::index_page_ownership`, and the internal `directory_drift_summary` helper from `unsafe fn` to safe functions.
- Converted the matching `src/am/mod.rs` IVF facade functions from `unsafe fn` to safe functions.
- Switched the SQL diagnostic wrappers for `ec_ivf_index_drift_snapshot`, `ec_ivf_index_admin_snapshot`, and `ec_ivf_index_page_ownership` from `with_live_index_relation!` to `with_live_index_relation_safe!`.

The remaining unsafe operations inside these functions are narrower PostgreSQL boundary reads, such as `RelationGetNumberOfBlocksInFork` and relcache field access, rather than whole-function unsafe contracts.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/114-ivf-admin-wrapper-boundaries unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1667` across `124` files.
- Ledger check passed: `ledger covers 1667 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
