# Task 50 Review Request: SPIRE Remote Candidate Result Boundary

## Summary

This slice narrows SPIRE remote-candidate coordinator helpers whose bodies now route through safe relation/view/store APIs.

Code commit: `0898b43a924f0b4c248d34406939ed761eed4222`

## Changes

- Converted remote-search candidate and coordinator-local candidate/result helpers from unsafe to safe where they no longer touch raw PostgreSQL pointers directly.
- Removed now-unnecessary unsafe blocks from local heap plan/candidate callers that only need the safe candidate result helper.
- Switched SQL wrappers for `ec_spire_remote_search`, `ec_spire_remote_search_coordinator_local`, and `ec_spire_remote_search_coordinator_local_summary` to `with_live_index_relation_safe!`.

Heap-resolution helpers that still cross the heap-fetch boundary remain unsafe for a later dedicated slice.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/121-spire-remote-candidate-result-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/121-spire-remote-candidate-result-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/121-spire-remote-candidate-result-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1641` across `124` files.
- Ledger check passed: `ledger covers 1641 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
