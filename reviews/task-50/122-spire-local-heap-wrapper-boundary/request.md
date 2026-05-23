# Task 50 Review Request: SPIRE Local Heap Wrapper Boundary

## Summary

This slice narrows SPIRE remote-search local heap wrappers after packet 121 made the candidate result helpers safe.

Code commit: `bdd090a526f1a682be949c36f87ae7f95dec912a`

## Changes

- Converted local heap resolution plan rows, local heap candidate rows, and local heap candidate summary helpers to safe APIs.
- Switched the corresponding SQL wrappers to `with_live_index_relation_safe!`.
- Removed an unnecessary unsafe block in the remote heap resolution pipeline summary.

The actual heap-fetch/materialization boundary remains inside `remote_search_local_heap_candidate_rows` and `remote_search_local_heap_candidate_rows_for_result_summary`; this slice only removes caller-side unsafe wrappers.

## Validation

- `git diff --check HEAD~1..HEAD`
- `make unsafe-block-count`
- `make UNSAFE_LEDGER=reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/122-spire-local-heap-wrapper-boundary unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Key results:

- Direct unsafe blocks dropped to `1638` across `123` files.
- Ledger check passed: `ledger covers 1638 current unsafe rows`.
- PG18 bench-feature compile check passed with the known existing SPIRE DML unused-import warning in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the packet-local artifact index.
