# Review Request: SPIRE Insert Publish Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/386-spire-insert-publish-boundary`
- code commit: `703bd0bd43fe93d05c33bfbc84dd819278fed334`
- scope: `src/am/ec_spire/insert.rs`

## Summary

This slice introduces a private `SpireInsertIndexRelation` boundary for SPIRE insert publication.

The wrapper now owns the publish lock guard and centralizes:

- acquiring the publish lock,
- reading the root/control state under that relation boundary,
- loading active local-store config and epoch manifests from the same root/control state.

The insert path now passes the wrapper's relation through the replacement publish path, making the lock/root-control/manifest relationship explicit and removing one direct unsafe block from `insert.rs`.

## Unsafe Count

- before this slice: `1149`
- after this slice: `1148`

## Validation

Artifacts are packet-local under `reviews/task-50/386-spire-insert-publish-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-insert.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1148`.
- `unsafe-ledger-generate.log`: generated 1148 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1148 current unsafe rows.
