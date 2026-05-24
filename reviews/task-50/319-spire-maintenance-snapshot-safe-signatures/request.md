# Task 50 Review Request: SPIRE Maintenance Snapshot Safe Signatures

## Summary

This slice converts the public SPIRE maintenance planning/run helpers from unsafe raw-relation entry points to safe `SpireLiveIndexRelation` helpers.

Code commit: `28d8b1a6a87d77bd5869fba453e1a553f83a6de0`

## What Changed

- Converted these helpers to take `SpireLiveIndexRelation`:
  - `index_maintenance_plan_snapshot`
  - `index_locked_maintenance_plan_snapshot`
  - `index_locked_maintenance_run_plan`
  - `index_maintenance_run`
- Updated SQL wrappers and scheduler wrappers in `src/lib.rs` to use `with_spire_live_index_relation!`.
- Reused `SpireLiveIndexRelation` methods for root-control, manifest, and object-store reads.
- Kept true low-level maintenance mutation work explicit inside `index_maintenance_run`, including heap-source reconstruction and publish calls.

## Review Focus

- Please verify these helpers now avoid the safe raw-pointer anti-pattern: public maintenance helpers require `SpireLiveIndexRelation`.
- Please check that retaining raw `index_relation` inside `index_maintenance_run` only for heap/publish internals remains the correct boundary.
- Please check publish-lock ordering remains unchanged for locked plan/run and actual run paths.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe maintenance signatures, maintenance SQL wrappers using the generic raw macro, and `checked_live_index_relation`
- `make UNSAFE_LEDGER=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1973` (down from packet 318 `1981`)
- Unsafe ledger rows: `1375`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-maintenance-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
