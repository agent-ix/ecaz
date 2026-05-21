# Task 50 Review Request: SPIRE Relation Store Boundary

## Summary

This slice removes redundant unsafe caller blocks around SPIRE relation-backed object store construction.

Code commit: `88c41377c4f3388ed1a9b3409f5e562a739c2c25`

## Changes

- Converted `SpireRelationObjectStore::for_index_relation` from `unsafe fn` to a safe constructor.
- Kept the relcache dereference as the single internal unsafe operation after explicit null and invalid-OID checks.
- Removed now-redundant unsafe call blocks from SPIRE maintenance planning/execution, active snapshot diagnostics, insert bootstrap, and debug roundtrip helpers.

This keeps the live-relation invariant at the SQL/AM caller boundary while avoiding repeated unsafe call wrappers for constructing the relation-store view.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/116-spire-relation-store-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/116-spire-relation-store-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/116-spire-relation-store-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1657` across `124` files.
- Ledger check passed: `ledger covers 1657 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
