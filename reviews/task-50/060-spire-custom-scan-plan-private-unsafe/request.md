# Task 50 Review Request: SPIRE Custom-Scan Plan-Private Unsafe

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates repeated unsafe blocks inside SPIRE custom-scan
plan-private metadata helpers:

- query-value expression classification for `Const` and `Param` nodes;
- float4 array constant decoding for plan-private query vectors.

The helpers remain `unsafe fn` because they inspect PostgreSQL planner
expression raw pointers. The cleanup keeps the null checks and NodeTag/type
guards, then uses one planner-expression unsafe region for the gated reads
instead of several scattered pointer and datum reads.

## Code

- commit: `6fbdd1fa Consolidate SPIRE custom scan plan-private unsafe`
- touched file: `src/am/ec_spire/custom_scan/plan_private.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/custom_scan/plan_private.rs` | 23 | 20 | -3 |
| `src/` ledger rows | 2155 | 2152 | -3 |

This does not close Task 50. The packet-local ledger still contains `2152`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2152` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves the same query-vector
eligibility rules: only float4 array constants and float4 array params are
accepted, null/empty/non-finite constants are rejected, and PostgreSQL planner
node tags continue to gate concrete expression reads.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

