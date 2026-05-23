# Review Request: SPIRE Publish Relation Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/385-spire-publish-relation-boundary`
- code commit: `e717b6f1eefd4bac870190cba7cb6d39b62fda5e`
- scope: `src/am/ec_spire/build/publish.rs`

## Summary

This slice adds a private `SpirePublishRelation` boundary for SPIRE publish-time relation writes.

The new wrapper centralizes:

- appending manifest bundle tuples,
- appending retired epoch manifests,
- appending placement entries,
- publishing root/control state after replacement epoch writes.

The change removes repeated call-site unsafe blocks around `page::append_object_tuple` and `page::initialize_root_control_page` while keeping the raw relation confined to the publish module's private boundary.

## Unsafe Count

- before this slice: `1154`
- after this slice: `1149`

## Validation

Artifacts are packet-local under `reviews/task-50/385-spire-publish-relation-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-build-publish.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1149`.
- `unsafe-ledger-generate.log`: generated 1149 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1149 current unsafe rows.
