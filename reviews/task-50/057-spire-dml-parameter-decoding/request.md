# Task 50 Review Request: SPIRE DML Parameter Decoding

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates SPIRE DML front-door executor parameter decoding:

- `ParamListInfo` null/bounds/fetch/direct-array access now copies
  `ParamExternData` fields inside one boundary, while any `paramFetch`
  workspace is still live.
- Integer `Datum` decoding now uses one typed boundary selected by the OID.

This removes repeated direct unsafe blocks while preserving the existing
executor parameter validation and bigint primary-key encoding behavior.

## Code

- commit: `7a0eb77f Consolidate SPIRE DML parameter decoding`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 63 | 58 | -5 |
| `src/` ledger rows | 2205 | 2200 | -5 |

This does not close Task 50. The packet-local ledger still contains `2200`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2200` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice keeps the same bigint parameter
validation and value encoding, but narrows the raw PostgreSQL parameter access
to one local copy boundary.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

