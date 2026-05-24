# Task 50 Review Request: SPIRE Coordinator Live Index View

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change continues the SPIRE live-index wrapper rollout in coordinator
snapshots. It removes local unsafe blocks that only re-called existing
coordinator snapshot wrappers or the module-local `live_index_relation`
boundary.

Touched paths include:

- storage, epoch, cleanup, and leaf snapshot setup;
- remote-node descriptor readiness;
- remote-node capability summaries;
- remote epoch publish and manifest readiness summaries.

The remaining explicit unsafe blocks in this file are the actual PostgreSQL
page, relation, timestamp, lock, and tuple mutation operations rather than
wrapper-only pass-throughs.

## Code

- commit: `fb2d528f Route SPIRE coordinator snapshots through live index view`
- touched file: `src/am/ec_spire/coordinator/snapshots.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/snapshots.rs` | 35 | 16 | -19 |
| `src/` ledger rows | 2125 | 2106 | -19 |

This does not close Task 50. The packet-local ledger still contains `2106`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2106` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves the same storage,
cleanup, leaf, remote-node, publish-gate, and manifest summary behavior while
reusing the existing live index relation boundary for SPIRE coordinator
diagnostic entry points.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

