# Review Request: Task 41 SPIRE Relation Store Open Guards

## Summary

This checkpoint wraps relation opens inside `src/am/ec_spire/storage/relation_store.rs`.

`SpireRelationObjectStoreSet` now stores `OpenedStoreRelation` guards instead of raw `(Relation, lockmode)` pairs. Both constructors open auxiliary store relations through `OpenedRelationsGuard::open`, retain the raw pointer only for `SpireRelationObjectStore`, and transfer the owning guards into the final store set. Partial-construction and final-drop paths both drain guards in reverse order.

## Safety Delta

- Baseline entries: `4325` -> `4321`.
- `src/am/ec_spire/storage/relation_store.rs` unsafe-comment baseline entries: `55` -> `51`.
- Direct `relation_open`/`relation_close` calls in this file are now isolated to `OpenedStoreRelation`.

## Reviewer Focus

- Confirm `OpenedRelationsGuard::into_inner` transfers ownership without closing successfully opened auxiliary relations.
- Confirm partial-construction error paths still close opened auxiliary relations in reverse order.
- Confirm `SpireRelationObjectStoreSet::drop` still closes final auxiliary relations in reverse order and never closes the root index relation passed by the caller.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see `artifacts/manifest.md`.
