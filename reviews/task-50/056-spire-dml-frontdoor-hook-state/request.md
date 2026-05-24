# Task 50 Review Request: SPIRE DML Front-Door Hook State

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change centralizes scattered DML front-door unsafe around:

- backend-local planner hook and relcache callback state snapshots;
- planner-hook observation status writes;
- chained planner invocation and plan-tree replacement dispatch;
- tuple descriptor attribute copying;
- catalog index relcache form reads;
- `format_type_be` string decoding and release.

The intent is to keep the unavoidable PostgreSQL global/hook/catalog boundary
unsafe in named local helpers while removing repeated caller-side unsafe blocks.

## Code

- commit: `10451b74 Centralize SPIRE DML frontdoor hook state`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 73 | 63 | -10 |
| `src/` ledger rows | 2215 | 2205 | -10 |

This does not close Task 50. The packet-local ledger still contains `2205`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2205` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice changes unsafe ownership around
backend-local hook state and catalog reads, but keeps the planner hook,
relcache invalidation, classifier diagnostics, catalog context construction,
and plan-tree replacement semantics intact.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

