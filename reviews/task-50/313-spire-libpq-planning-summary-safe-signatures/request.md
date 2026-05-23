# Review Request: SPIRE Libpq Planning Summary Safe Signatures

## Summary

This checkpoint removes unnecessary unsafe boundaries from SPIRE libpq planning
summary helpers and the shared remote candidate index OID helper.

`remote_candidate_index_oid` now owns the live relation view construction and is
safe for callers that already operate under the SQL wrapper's
`IndexRelationGuard`. Libpq request, connection, dispatch, and executor budget
summary helpers are now safe rollups over existing planning rows. SQL wrapper
call sites now use `with_live_index_relation_safe!` for those helpers.

## Code Commit

- `140f94ac06e26caca2befaddaac05796619bccae` - `Make SPIRE libpq planning summaries safe`

## Unsafe Count

- Previous packet baseline after packet 312: `2038`
- After this checkpoint: `2025`
- Net change: `-13`
- `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` by-file match count: `1`
- `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` by-file match count: `7`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs` by-file match count: `7`
- `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs` by-file match count: `1`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1375 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/313-spire-libpq-planning-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/313-spire-libpq-planning-summary-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/313-spire-libpq-planning-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
