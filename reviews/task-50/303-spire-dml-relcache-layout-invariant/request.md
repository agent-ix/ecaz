# Review Request: SPIRE DML Relcache Layout Invariant

## Summary

This checkpoint responds to the soundness audit recommendation in
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-21-01-reviewer.md`
for the SPIRE DML frontdoor relcache read at `dml_frontdoor/mod.rs:1510-1543`.

The SAFETY comment now names the PG17+ relcache layout dependency: opened index
relations are expected to expose initialized `rd_index` and `rd_rel` entries in
PostgreSQL `RelationData`. This makes the ABI/layout coupling explicit for
future PostgreSQL-version audits.

This is documentation-only and intentionally does not change behavior.

## Code Commit

- `8fd9af0c817639e601e756efb2ea9eea37b70c30` - `Document SPIRE DML relcache layout invariant`

## Unsafe Count

- Previous packet baseline after packet 302: `2061`
- After this checkpoint: `2061`
- Net change: `0`
- `src/am/ec_spire/dml_frontdoor/mod.rs` by-file match count remains `64`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/303-spire-dml-relcache-layout-invariant/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/303-spire-dml-relcache-layout-invariant unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/303-spire-dml-relcache-layout-invariant/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
