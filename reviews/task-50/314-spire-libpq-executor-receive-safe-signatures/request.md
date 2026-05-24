# Review Request: SPIRE Libpq Executor Receive Safe Signatures

## Summary

This checkpoint removes unnecessary unsafe signatures from SPIRE libpq executor
receive helpers and dependent summary paths.

The executor candidate, receive-attempt, heap-candidate, identity-cache,
receive-plan, and merge-input helpers now expose safe signatures. Their callers
in SQL wrappers, coordinator result summaries, finalization summaries, and the
pg_test identity-cache fixture now call them through safe APIs while retaining
the existing relation guard and planning checks.

## Code Commit

- `83e39b5373cf442d514aba9a124bb235a6587f75` - `Make SPIRE libpq executor receive helpers safe`

## Unsafe Count

- Previous packet baseline after packet 313: `2025`
- After this checkpoint: `2015`
- Net change: `-10`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` by-file match count: `23`
- `src/tests/remote_search/libpq_executor.rs` by-file match count: `0`
- `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` by-file match count: `0`
- `src/am/ec_spire/coordinator/remote_candidates/result_contracts.rs` by-file match count: `0`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1372 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/314-spire-libpq-executor-receive-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/314-spire-libpq-executor-receive-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/314-spire-libpq-executor-receive-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
