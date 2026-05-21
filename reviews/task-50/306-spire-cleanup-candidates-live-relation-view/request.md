# Review Request: SPIRE Cleanup Candidates Live Relation View

## Summary

This checkpoint narrows the SPIRE epoch-cleanup candidate scan from raw relation
plumbing to typed live relation views.

`collect_physical_cleanup_candidates` now takes `SpireLiveIndexRelation`
instead of a raw `pg_sys::Relation`, removing an unnecessary `unsafe fn`
boundary. Storage/index relations selected during cleanup are represented as
`SpireLiveObjectRelation`, which exposes a safe tuple-scan method while keeping
the underlying page scan unsafe centralized in one relation-view method.

The first code commit made the opener safe; the second commit completed the
view refactor so the packet does not add a new ledger row for an extra local
unsafe block.

## Code Commits

- `3c0417d9ee5b2c3b95404d8a323b2cc205d711cc` - `Bind SPIRE cleanup candidate scan to live relation view`
- `bfb5b96d42fdd53318a9434e72397bbb75a01087` - `Route SPIRE cleanup scans through object relation view`

## Unsafe Count

- Previous packet baseline after packet 305: `2060`
- After this checkpoint: `2059`
- Net change: `-1`
- `src/am/ec_spire/coordinator/snapshots.rs` by-file match count: `35`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`
- Note: this slice removes an `unsafe fn` marker and keeps the direct page-scan
  unsafe centralized in the relation-view method, so the direct unsafe ledger
  row count remains unchanged.

## Validation

- `git diff --check HEAD~2..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/306-spire-cleanup-candidates-live-relation-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/306-spire-cleanup-candidates-live-relation-view unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/306-spire-cleanup-candidates-live-relation-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
