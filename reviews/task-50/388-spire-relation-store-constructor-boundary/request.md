# Review Request: SPIRE Relation Store Constructor Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/388-spire-relation-store-constructor-boundary`
- code commit: `6d7c301eba16d4baa91987870710a664e4deb8ce`
- scope: `src/am/ec_spire/storage/relation_store.rs`

## Summary

This slice adds a private `SpireRelationObjectStore::from_live_store_relation` constructor boundary.

The store-set builders now route repeated `for_store_relation_id` construction through that private boundary. This keeps the relation lifetime invariant local to the storage module:

- store relations are either the already-open SPIRE index relation or relations owned by `OpenedRelationsGuard`;
- constructed stores do not outlive the owning relation guard/store set.

No new safe public raw-relation API was added.

## Unsafe Count

- before this slice: `1137`
- after this slice: `1136`

## Validation

Artifacts are packet-local under `reviews/task-50/388-spire-relation-store-constructor-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-relation-store.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1136`.
- `unsafe-ledger-generate.log`: generated 1136 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1136 current unsafe rows.
