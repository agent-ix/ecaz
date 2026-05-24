# Review Request: SPIRE Remote Candidate Summary Safe Signatures

## Summary

This checkpoint removes unnecessary `unsafe fn` boundaries from SPIRE remote
candidate summary helpers that either compose safe planning helpers or already
isolate their raw relation operation internally.

The SQL wrapper call sites in `src/lib.rs` now use the existing
`with_live_index_relation_safe!` macro for these helpers, avoiding new
`unused_unsafe` warnings while keeping the `IndexRelationGuard` lifetime check
at the wrapper boundary.

## Code Commit

- `760fcd3fe440b1c88e8d34c3eaae018f9bdd8a3f` - `Make SPIRE remote candidate summary helpers safe`

## Unsafe Count

- Previous packet baseline after packet 310: `2045`
- After this checkpoint: `2041`
- Net change: `-4`
- `src/am/ec_spire/coordinator/remote_candidates/fault_matrix.rs` by-file match count: `1`
- `src/am/ec_spire/coordinator/remote_candidates/result_contracts.rs` by-file match count: `1`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1383 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/311-spire-remote-candidate-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/311-spire-remote-candidate-summary-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/311-spire-remote-candidate-summary-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

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
