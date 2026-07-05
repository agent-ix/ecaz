# Task 50 Review Request: SPIRE Relation Object Reader Safe Facade

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change makes the relation-backed SPIRE object reader methods safe on
the typed `SpireRelationObjectStore` and `SpireRelationObjectStoreSet` facade.
The raw relation lifetime boundary remains at object-store construction. The
read methods already validate placement/store identity, pin tuple bytes only
inside callback scopes, and copy/decode owned objects before returning.

This removes repeated caller unsafe from:

- relation-store trait delegation;
- relation-store set trait delegation;
- active object tuple locator lookup;
- debug roundtrip helpers;
- coordinator storage and hierarchy snapshot callers.

## Code

- commit: `694a4bf5 Make SPIRE relation object reads safe`
- touched files:
  - `src/am/ec_spire/storage/relation_store.rs`
  - `src/am/ec_spire/coordinator/debug.rs`
  - `src/am/ec_spire/coordinator/snapshots.rs`
  - `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/storage/relation_store.rs` | 38 | 16 | -22 |
| `src/am/ec_spire/coordinator/debug.rs` | 38 | 36 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 37 | 35 | -2 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 38 | 37 | -1 |
| `src/` ledger rows | 2200 | 2173 | -27 |

This does not close Task 50. The packet-local ledger still contains `2173`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2173` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice changes API safety ownership around
the existing typed object-store facade; tuple pinning, placement validation,
object decoding, and store-set relation ownership are unchanged.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

