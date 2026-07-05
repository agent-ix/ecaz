# Task 50 Review Request: SPIRE DML Query-Shape Unsafe

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates repeated unsafe blocks inside SPIRE DML frontdoor
query-shape helpers:

- DML jointree/fromlist inspection;
- single range-table-ref extraction;
- range-table OID lookup;
- baserestrictinfo predicate extraction;
- `OpExpr` predicate decoding;
- implicit-coercion one-argument list reads;
- target-list column/value extraction.

The helper boundaries stay honest about PostgreSQL raw planner memory: no safe
facade was introduced for arbitrary raw pointers. Each consolidation keeps the
same live planner-tree precondition and groups related read-only Node/List/C
string reads into one local unsafe region.

## Code

- commit: `5fe9abcf Consolidate SPIRE DML query shape unsafe`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 58 | 47 | -11 |
| `src/` ledger rows | 2152 | 2141 | -11 |

This does not close Task 50. The packet-local ledger still contains `2141`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2141` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves the existing shape
classifier decisions for joins, range table lookup, PK predicates, coercion
wrappers, and target-list column extraction. The `OpExpr` path still reports PK
column references with unsupported operators as `operator: other`, matching the
previous fail-closed behavior.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

