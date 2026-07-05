# Review Request: SPIRE Remote Candidate Result Live Relation View

## Summary

This checkpoint binds three SPIRE remote candidate result helpers in
`src/am/ec_spire/coordinator/hierarchy_snapshots.rs` to `SpireLiveIndexRelation`.

The public `unsafe` PG callback-facing entry points still construct the typed
view at the raw-relation boundary. The private result helpers are now safe
functions that receive the live relation view instead of a raw
`pg_sys::Relation`, avoiding the reviewed anti-pattern of safe raw-pointer
deref helpers with unbounded returned references.

## Code Commit

- `6df23614f6256a6fb4f90bea13cbf94a5e7f77e1` - `Bind SPIRE remote candidate result helpers to live relation view`

## Unsafe Count

- Previous packet baseline after packet 307: `2057`
- After this checkpoint: `2054`
- Net change: `-3`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` by-file match count: `30`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1388 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/308-spire-remote-candidate-results-live-relation-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/308-spire-remote-candidate-results-live-relation-view unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/308-spire-remote-candidate-results-live-relation-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
