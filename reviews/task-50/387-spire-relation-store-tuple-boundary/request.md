# Review Request: SPIRE Relation Store Tuple Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/387-spire-relation-store-tuple-boundary`
- code commit: `d780f21519405af4c8641f4f283eef0f604f33a5`
- scope: `src/am/ec_spire/storage/relation_store.rs`

## Summary

This slice adds a private `SpireRelationObjectStore::with_object_tuple` boundary for relation-backed tuple reads.

The wrapper centralizes the pinned-object tuple callback contract for:

- routing object chain-meta probes,
- leaf V2 meta and segment reads,
- object header reads,
- active tuple locator collection,
- top graph chain-meta probes,
- raw object byte reads,
- large partition object segment reads.

The change removes repeated call-site unsafe blocks around `page::with_pinned_object_tuple` and keeps tuple bytes scoped to the callback while the page remains pinned.

## Unsafe Count

- before this slice: `1148`
- after this slice: `1137`

## Validation

Artifacts are packet-local under `reviews/task-50/387-spire-relation-store-tuple-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-relation-store.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1137`.
- `unsafe-ledger-generate.log`: generated 1137 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1137 current unsafe rows.
