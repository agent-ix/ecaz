# Task 50 Review Request: SPIRE Live Relation View Boundary

## Summary

This slice removes the unsafe constructor contract around the shared SPIRE coordinator live-relation view.

Code commit: `cd9e10744a6b7e8099bfe7f5869db14d6dbe1239`

## Changes

- Converted `SpireLiveIndexRelation::new`, `live_index_relation`, and `live_index_relid` to safe helpers.
- Added a null-relation guard at `live_index_relation`.
- Kept actual relcache/page reads inside the typed view methods where their PostgreSQL boundary comments already live.

This reduces a shared coordinator relation-view unsafe boundary without changing remote candidate or snapshot behavior.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/120-spire-live-relation-view-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/120-spire-live-relation-view-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/120-spire-live-relation-view-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1647` across `124` files.
- Ledger check passed: `ledger covers 1647 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
