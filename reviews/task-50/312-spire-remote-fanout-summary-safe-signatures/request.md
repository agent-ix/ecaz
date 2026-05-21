# Review Request: SPIRE Remote Fanout Summary Safe Signatures

## Summary

This checkpoint removes unnecessary `unsafe fn` boundaries from SPIRE remote
fanout and fanout-summary helpers.

`remote_search_fanout_plan_rows`, `remote_search_request_summary_row`, and
`remote_search_readiness_summary_row` are now safe helpers. The fanout and
target planning root-control reads now go through `SpireLiveIndexRelation`
instead of direct raw-relation page reads. The SQL wrapper call sites now use
`with_live_index_relation_safe!` while retaining the same `IndexRelationGuard`
validation and lifetime scope.

## Code Commit

- `dd80e7d8456fa975037dc181c2e513b182f94bf4` - `Make SPIRE remote fanout summaries safe`

## Unsafe Count

- Previous packet baseline after packet 311: `2041`
- After this checkpoint: `2038`
- Net change: `-3`
- `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` by-file match count: `3`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1383 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/312-spire-remote-fanout-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/312-spire-remote-fanout-summary-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/312-spire-remote-fanout-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
