# Task 50 Review Request: SPIRE Relation Plan Boundary

## Summary

This slice narrows SPIRE build-time local-store relation planning by removing an unsafe helper call boundary.

Code commit: `256b45d65e01bfa05c9279add7673e37c9e86dc3`

## Changes

- Converted `spire_aux_store_reloptions` from `unsafe fn` to a safe helper with its PostgreSQL allocation/datum conversions contained internally.
- Converted `create_local_store_relations_for_build` from `unsafe fn` to a safe helper with explicit null and invalid-relid validation before relcache reads.
- Removed the now-unnecessary unsafe call block around `spire_aux_store_reloptions`.

The remaining unsafe in `relation_plan.rs` is localized around PostgreSQL relcache/catalog APIs and carries the existing per-call `SAFETY` comments.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/117-spire-relation-plan-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/117-spire-relation-plan-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/117-spire-relation-plan-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1656` across `124` files.
- Ledger check passed: `ledger covers 1656 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
