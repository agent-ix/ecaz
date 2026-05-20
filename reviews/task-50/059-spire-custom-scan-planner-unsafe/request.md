# Task 50 Review Request: SPIRE Custom-Scan Planner Unsafe

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates repeated unsafe blocks inside SPIRE custom-scan
planner helpers:

- remote custom-scan candidate discovery;
- DML PK-select candidate discovery;
- vector-order custom path construction;
- DML PK-select custom path construction.

The helpers remain `unsafe fn` because they operate on PostgreSQL planner-hook
raw pointers. The cleanup narrows each helper to one named planner-boundary
unsafe region instead of scattered pointer/list reads and planner-memory
allocations.

## Code

- commit: `1de12ef0 Consolidate SPIRE custom scan planner unsafe`
- touched file: `src/am/ec_spire/custom_scan/planner.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/custom_scan/planner.rs` | 37 | 19 | -18 |
| `src/` ledger rows | 2173 | 2155 | -18 |

This does not close Task 50. The packet-local ledger still contains `2155`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2155` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves the same planner hook
candidate checks, catalog placement lookup, path costs, path private lists, and
`add_path` behavior while centralizing the raw PostgreSQL planner boundary.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

