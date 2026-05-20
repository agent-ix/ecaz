# Task 50 Review Request: SPIRE Hierarchy Live Index View

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change routes SPIRE hierarchy and remote-search diagnostic entry
points through the existing `SpireLiveIndexRelation` wrapper via
`live_index_relation` instead of constructing the live index view with repeated
local unsafe blocks.

Touched paths include:

- insert-debt snapshot setup;
- remote candidate and coordinator-local candidate summaries;
- top-graph, hierarchy, object, delta, placement, routing, and centroid
  diagnostic snapshots.

The wrapper is already the module-local representation of "PostgreSQL supplied
a live SPIRE index relation for this diagnostic call"; this slice reuses that
single boundary consistently instead of restating it at each call site.

## Code

- commit: `70a8ff9f Route SPIRE hierarchy diagnostics through live index view`
- touched file: `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 37 | 21 | -16 |
| `src/` ledger rows | 2141 | 2125 | -16 |

This does not close Task 50. The packet-local ledger still contains `2125`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2125` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves the same active epoch,
manifest, object-store, routing, placement, and centroid diagnostic reads while
reusing the live relation view that already owns the relation-lifetime
invariant in the included SPIRE coordinator snapshot module.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

