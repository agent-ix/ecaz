# Review Request: Task 41 SPIRE Coordinator Debug Index Guards

## Summary

This checkpoint wraps manual index open/close handling in `src/am/ec_spire/coordinator/debug.rs`.

The `pg_test`/test-only SPIRE coordinator debug helpers now use a `DebugIndexRelation` guard that stores the lockmode and owns the matching `index_close`. The helpers pass `as_ptr()` into the existing page, storage, manifest, and scan routines, preserving the old lockmode choices while removing explicit close calls from the helper bodies.

## Safety Delta

- Baseline entries: `4347` -> `4325`.
- `src/am/ec_spire/coordinator/debug.rs` unsafe-comment baseline entries: `60` -> `38`.
- Direct `index_open`/`index_close` calls in this file are now isolated to `DebugIndexRelation`.

## Reviewer Focus

- Confirm RowExclusive debug mutation helpers keep the guard alive across all page/storage writes.
- Confirm `debug_spire_relation_two_store_scan_roundtrip` keeps both root and aux guards alive while the store set and candidate collection use them.
- Confirm AccessShare read-only helpers preserve their previous lockmode and return owned data before drop.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see `artifacts/manifest.md`.
