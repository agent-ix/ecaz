# Task 50 Review Request: SPIRE Hierarchy Snapshot Safe Signatures

## Summary

This slice continues the SPIRE unsafe burndown after packet 316 by converting the remaining hierarchy diagnostic helpers from raw `pg_sys::Relation` unsafe entry points to safe helpers over `SpireLiveIndexRelation`.

Code commit: `c07a7dd5fae5e0ff80ad371956807333805728d2`

## What Changed

- Converted these SPIRE hierarchy helpers from `pub(crate) unsafe fn(... pg_sys::Relation ...)` to safe `pub(crate) fn(... SpireLiveIndexRelation ...)`:
  - `index_top_graph_snapshot`
  - `index_hierarchy_snapshot`
  - `index_object_snapshot`
  - `index_delta_snapshot`
  - `index_scan_placement_snapshot`
  - `index_selected_pid_placement_snapshot`
  - `index_scan_routing_snapshot`
  - `index_root_routing_snapshot`
  - `index_routing_centroid_snapshot`
  - `classify_centroid`
- Updated SQL wrappers in `src/lib.rs` to use `with_spire_live_index_relation!` for these helpers.
- Updated the SPIRE cost helper to construct `SpireLiveIndexRelation` at the AM callback boundary before calling the safe hierarchy snapshot helper.
- Reworked selected-PID placement snapshot loading to use the existing live-index fanout anchor rather than re-reading manifests through a raw relation pointer.

## Review Focus

- Please verify this does not repeat the packet 311-315 anti-pattern: the safe helpers now require `SpireLiveIndexRelation`; they do not accept raw `pg_sys::Relation` while hiding a relation dereference.
- Please check that the SQL wrapper and AM cost callback boundaries are the right places to construct the live relation wrapper.
- Please check the selected-PID placement snapshot behavior remains equivalent for inactive indexes and active fanout manifests.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for the removed unsafe hierarchy signatures and `checked_live_index_relation`
- `make UNSAFE_LEDGER=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1993` (down from packet 316 `2003`)
- Unsafe ledger rows: `1382`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-hierarchy-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
