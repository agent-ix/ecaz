# Task 50 Review Request: SPIRE Hierarchy Live Relation View

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change rolls four SPIRE hierarchy diagnostic/classification paths
through the existing `SpireLiveIndexRelation` facade:

- `index_scan_routing_snapshot`
- `index_root_routing_snapshot`
- `index_routing_centroid_snapshot`
- `classify_centroid`

Those paths previously reopened the same relation-level unsafe operations at
each caller: root/control reads, epoch manifest loading, object-store opening,
relation option access, and direct active-manifest object tuple reads. They now
construct the live relation view once at the PostgreSQL boundary and use safe
methods on that view for the repeated operations.

## Code

- commit: `2b759d6c Roll SPIRE hierarchy snapshots through live relation view`
- touched file: `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 48 | 38 | -10 |
| `src/` ledger rows | 2225 | 2215 | -10 |

This does not close Task 50. The packet-local ledger still contains `2215`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2215` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This is a relation-view cleanup over existing
diagnostic/classification read paths. It changes where unsafe is owned, not the
snapshot construction, routing diagnostics, root/routing-centroid collection, or
classification algorithms.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

