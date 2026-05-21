# Review Request: SPIRE Live Index Relid Helper Removal

## Summary

This checkpoint removes the redundant `live_index_relid` helper from
`src/am/ec_spire/coordinator/snapshots.rs`.

The only caller already had a `SpireLiveIndexRelation` view in scope, so it now
uses `index.relid()` directly. This deletes a raw-relation unsafe helper and its
internal unsafe block without introducing a safe raw-pointer dereference API.

## Code Commit

- `ebe20de3f688c398b1c4e84f7270f7b9f0afa956` - `Remove redundant SPIRE live index relid helper`

## Unsafe Count

- Previous packet baseline after packet 306: `2059`
- After this checkpoint: `2057`
- Net change: `-2`
- `src/am/ec_spire/coordinator/snapshots.rs` by-file match count: `33`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1388 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/307-spire-live-index-relid-helper-removal/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/307-spire-live-index-relid-helper-removal unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/307-spire-live-index-relid-helper-removal/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
