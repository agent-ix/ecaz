# Review Request: SPIRE Snapshot Epoch Manifest Wrapper

## Summary

This checkpoint responds to the soundness audit recommendation in
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-21-01-reviewer.md`
for the SPIRE coordinator snapshot path.

`SpireLiveIndexRelation::active_epoch_anchor` no longer calls the scan-layer
`load_relation_epoch_manifests` unsafe helper directly. The call is now routed
through a private `SpireLiveIndexRelation::load_active_epoch_manifests` method,
so the live-relation/root-control contract is named on the relation view and
`active_epoch_anchor` remains a safe composition step.

This is a structural cleanup and intentionally does not change behavior.

## Code Commit

- `27bb80312d0c6bbf2b82e4ec68f5bb61ffb162ae` - `Wrap SPIRE snapshot epoch manifest load`

## Unsafe Count

- Previous packet baseline after packet 301: `2061`
- After this checkpoint: `2061`
- Net change: `0`
- `src/am/ec_spire/coordinator/snapshots.rs` by-file match count remains `37`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/302-spire-snapshot-epoch-manifest-wrapper/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/302-spire-snapshot-epoch-manifest-wrapper unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/302-spire-snapshot-epoch-manifest-wrapper/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

`cargo check` passes. The log includes the known pre-existing SPIRE unused-import
warning in `src/am/mod.rs`.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
