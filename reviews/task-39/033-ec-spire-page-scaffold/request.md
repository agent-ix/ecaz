# Task 39 ec_spire/page.rs shadow-crate scaffold

## Summary

Lifts `am/ec_spire/page.rs` off **0.00%** coverage by mounting it
inside the `hardening/careful` shadow crate. New baseline: **11.01%**
(line) / 13.33% (function) / 5.87% (region).

This is the explicit post-RaBitQ pivot direction from the
2026-05-19 handoff ("pivot to `ec_spire/page.rs` coverage from 0%").

The scaffold is intentionally minimal — it stubs the pg_sys
page/WAL surface so the file compiles, and adds 6 tests against
early-error paths that return `Err` before any real page operation.
Raising coverage further (the encode/decode success paths) needs a
backing-page emulator and is a separate slice.

## Code under review

- Code scaffold commit: `f360f4264bd1dec6e3a0d04b6ff20bb02b4f1320`
- Baseline ratchet commit: pending (this packet only edits
  `fixtures/quality/coverage-baseline.tsv`).
- Changed files: `hardening/careful/src/pg_guards.rs`,
  `hardening/careful/src/lib.rs`,
  `hardening/careful/src/spire.rs`,
  `fixtures/quality/coverage-baseline.tsv`.

## Scaffold details

Added to `hardening/careful/src/pg_guards.rs`:

- Constants: `RowExclusiveLock`, `BUFFER_LOCK_SHARE`,
  `BUFFER_LOCK_EXCLUSIVE`, `GENERIC_XLOG_FULL_IMAGE`,
  `InvalidOffsetNumber`.
- Types: `XLogRecPtr`, `GenericXLogState`, `ItemIdData`
  (`lp_off` / `lp_flags` / `lp_len` getters), `ItemId`.
- Relation/FSM stubs: `RelationGetNumberOfBlocksInFork`
  (configurable per-thread via `set_relation_block_count`),
  `GetPageWithFreeSpace`, `RecordPageWithFreeSpace`.
- GenericXLog stubs: `GenericXLogStart`,
  `GenericXLogRegisterBuffer`, `GenericXLogFinish`,
  `GenericXLogAbort`.
- Page-level no-op stubs: `PageInit`, `PageAddItemExtended`,
  `PageGetItem`, `PageGetItemId`, `PageGetMaxOffsetNumber`,
  `PageGetFreeSpace`, `PageGetSpecialPointer`,
  `PageGetSpecialSize`, `PageIndexTupleDeleteNoCompact`.

Wires the existing `buffer_guard` and a new `wal` shadow module
into `crate::storage` (so `page.rs`'s `use crate::storage::{...}`
resolves) and mounts `src/am/ec_spire/page.rs` as
`careful_spire::page`.

## Tests

Six new tests in `careful_spire::page_tests`:

- `read_object_tuple_rejects_metadata_block_tid`
- `append_object_tuple_rejects_empty_payload`
- `append_object_tuple_rejects_uninitialized_relation` (forces
  `RelationGetNumberOfBlocksInFork == 0` via the per-thread test
  hook)
- `delete_object_tuples_no_compact_is_noop_for_empty_input`
- `delete_object_tuples_no_compact_rejects_metadata_block_tid`
- `first_data_block_is_immediately_after_metadata_block` pins the
  `METADATA_BLOCK_NUMBER == 0` / `FIRST_DATA_BLOCK_NUMBER == 1`
  invariant that the guards rely on.

All six target paths that return `Err` before any page-level
`pg_sys` call, so the no-op page stubs are never executed.

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`:
  **461 passed** (was 455 before this packet).
- `make coverage`: `ec_spire/page.rs` 0.00 → **11.01** line
  coverage (`artifacts/coverage/summary.txt`).
- `scripts/check_coverage_delta.sh` with the single-path
  changed-files list — `coverage ok: am/ec_spire/page.rs
  actual=11.01 baseline=11.01`. Artifact:
  `artifacts/coverage-delta-check.log`.
- `scripts/check_coverage_baseline_complete.sh` — `coverage
  baseline complete for 40 critical paths`. Artifact:
  `artifacts/coverage-baseline-check.log`.

## Notes / follow-ups

- The remaining 89% of `page.rs` lines need a backing-page
  emulator inside the pg_sys mocks (real `PageInit` /
  `PageAddItemExtended` semantics with a per-buffer 8K backing
  buffer and ItemId table). That's a multi-hour slice in itself.
- The same pg_sys page-stub surface unblocks
  `ec_spire/storage/relation_store.rs` (also 0%) once a
  backing-page emulator is in place, since `relation_store.rs`
  shares the same `LockedBufferGuard` + page-add flow.
