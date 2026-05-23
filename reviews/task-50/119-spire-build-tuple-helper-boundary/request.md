# Task 50 Review Request: SPIRE Build Tuple Helper Boundary

## Summary

This slice narrows SPIRE build and insert tuple decoding by moving helper-level safety checks into safe APIs.

Code commit: `6d417d6aca10fab9a3e675246930a65aa6f75aae`

## Changes

- Converted `decode_heap_tid` from `unsafe fn` to a safe helper with the existing null check and localized pointer copy.
- Converted `resolve_indexed_tuple_layout` from `unsafe fn` to a safe helper and added an explicit null heap-relation check before relcache access.
- Converted the internal type-kind helpers to safe helpers, with PostgreSQL type lookup unsafe kept inside those helpers.
- Removed the now-redundant unsafe blocks from the SPIRE insert path.

The remaining unsafe in this area is still tied to actual PostgreSQL datum, tuple descriptor, and callback pointer operations.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/119-spire-build-tuple-helper-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/119-spire-build-tuple-helper-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/119-spire-build-tuple-helper-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1648` across `124` files.
- Ledger check passed: `ledger covers 1648 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
