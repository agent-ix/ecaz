# Task 50 Review Request: SPIRE Remote Epoch Snapshot Safe Signatures

## Summary

This slice continues the SPIRE unsafe burndown by converting remote node capability/readiness and remote epoch/manifest planning helpers to safe `SpireLiveIndexRelation` signatures.

Code commit: `7f05b7d42c0eddcbb8efb3f9804b6ca7498640df`

## What Changed

- Converted these helpers from unsafe raw-relation entry points to safe `SpireLiveIndexRelation` entry points:
  - `remote_node_descriptor_readiness`
  - `remote_node_descriptor_readiness_summary`
  - `remote_node_capability_plan`
  - `remote_node_capability_summary`
  - `remote_epoch_publish_plan`
  - `remote_epoch_publish_readiness`
  - `remote_epoch_publish_gate_summary`
  - `remote_epoch_manifest_plan`
  - `remote_epoch_manifest_summary`
- Updated SQL wrappers and compound manifest flows in `src/lib.rs` to construct the SPIRE live relation wrapper at the validated guard boundary.
- Updated `remote_search_operator_diagnostics_row` to construct `SpireLiveIndexRelation` once at its existing unsafe raw callback boundary and pass that wrapper to `remote_node_capability_summary`.

## Review Focus

- Please verify the remote node/epoch helpers now avoid the packet 311-315 anti-pattern: safe helpers require `SpireLiveIndexRelation`, not raw `pg_sys::Relation`.
- Please check that compound manifest flows reuse the validated SQL guard boundary correctly.
- Please check that `remote_search_operator_diagnostics_row` remains a correct unsafe raw boundary and does not duplicate live relation construction.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe remote node/epoch signatures and `checked_live_index_relation`
- `make UNSAFE_LEDGER=reviews/task-50/318-spire-remote-epoch-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/318-spire-remote-epoch-snapshot-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/318-spire-remote-epoch-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1981` (down from packet 317 `1993`)
- Unsafe ledger rows: `1379`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-remote-epoch-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
