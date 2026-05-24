# Review Request: SPIRE Heap Candidate Summaries Live Relation View

## Summary

This checkpoint binds the SPIRE local heap-candidate summary helpers to
`SpireLiveIndexRelation`.

The public SQL/AM-facing entry points still receive raw `pg_sys::Relation`
values and construct the typed view at that boundary. Private helpers now
accept the typed live-relation view, so the local heap candidate and coordinator
summary paths no longer expose private `unsafe fn` wrappers just to pass raw
relations through the call chain. The remaining unsafe operations are the
specific heap attribute resolution, heap slot reader construction, and remote
executor call sites that still require PostgreSQL raw-pointer contracts.

## Code Commit

- `6bc654ccf95a2bf637662ec8e2b9961b9bce0d13` - `Bind SPIRE heap candidate summaries to live relation view`

## Unsafe Count

- Previous packet baseline after packet 308: `2054`
- After this checkpoint: `2048`
- Net change: `-6`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` by-file match count: `24`
- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` by-file match count: `6`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1386 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/309-spire-heap-candidate-summaries-live-relation-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/309-spire-heap-candidate-summaries-live-relation-view unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/309-spire-heap-candidate-summaries-live-relation-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
