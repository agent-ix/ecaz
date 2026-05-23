# Task 50 Review Request: SPIRE Store Set Boundary

## Summary

This slice removes repeated unsafe caller blocks around SPIRE relation object store-set construction.

Code commit: `6421e4e8a2630b0b17e0f47b0daf7a14c93cd12e`

## Changes

- Converted `SpireRelationObjectStoreSet::for_index_relation_and_config` from `unsafe fn` to a safe constructor.
- Kept the relcache OID read localized inside the constructor after an explicit null-relation check.
- Removed unsafe wrappers from SPIRE insert, vacuum, single-level build publish, and recursive build publish call sites.

The constructor owns any auxiliary relation guards it opens, so callers do not need to repeat unsafe relation-store setup blocks.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/118-spire-store-set-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/118-spire-store-set-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/118-spire-store-set-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1652` across `124` files.
- Ledger check passed: `ledger covers 1652 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
