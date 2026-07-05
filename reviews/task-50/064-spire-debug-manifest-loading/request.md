# Task 50 Review Request: SPIRE Debug Manifest Loading

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates repeated pg_test-only SPIRE debug manifest loading
into one helper:

- local-store config load;
- epoch manifest load;
- object manifest load;
- placement directory load.

The placement-state, placement-node, and consistency-mode debug rewrite helpers
now reuse that single manifest-bundle boundary instead of hand-reading the same
root-control referenced tuples. The empty-manifest roundtrip also copies the
debug relation OID/tablespace in one guarded read.

## Code

- commit: `9258d294 Consolidate SPIRE debug manifest loading`
- touched file: `src/am/ec_spire/coordinator/debug.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/debug.rs` | 36 | 29 | -7 |
| `src/` ledger rows | 2106 | 2099 | -7 |

This does not close Task 50. The packet-local ledger still contains `2099`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2099` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice keeps the same pg_test debug
manifest rewrite semantics while routing repeated tuple loads through one
root-control-anchored helper.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

