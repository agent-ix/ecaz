# Review Request: SPIRE Storage Relation Opener Safe

## Summary

This checkpoint removes an unnecessary `unsafe fn` marker from
`open_storage_relation_or_index` in `src/am/ec_spire/coordinator/snapshots.rs`.

The helper does not dereference the raw `index_relation` pointer. It compares
relation OIDs, returns the already-open index relation when the storage relation
is the index itself, or opens a separate storage relation through
`RelationGuard`. The actual PostgreSQL page/relation unsafe remains at the
callers that read block counts, scan object tuples, or delete tuples.

This advances the SPIRE coordinator residual minimization from Wave 4 while
preserving the rule that raw-pointer-to-reference helpers stay unsafe or
closure-bound.

## Code Commit

- `25898a1ffb3432fd46bfc445fc06ab953b8bd3c2` - `Make SPIRE storage relation opener safe`

## Unsafe Count

- Previous packet baseline after packet 304: `2061`
- After this checkpoint: `2060`
- Net change: `-1`
- `src/am/ec_spire/coordinator/snapshots.rs` by-file match count: `36`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`
- Note: this slice removes an `unsafe fn` marker, while the ledger tracks
  current direct unsafe rows/blocks; therefore the ledger row count is unchanged.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/305-spire-storage-relation-opener-safe/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/305-spire-storage-relation-opener-safe unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/305-spire-storage-relation-opener-safe/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
