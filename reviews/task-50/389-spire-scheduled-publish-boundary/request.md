# Review Request: SPIRE Scheduled Publish Boundary

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/389-spire-scheduled-publish-boundary`
- code commit: `a06565b71b72dd0041d1ba1b2cbc56bfbe24b68a`
- scope: `src/am/ec_spire/update/publish/relation.rs`

## Summary

This slice adds a private `SpireRelationScheduledPublishRelation` boundary for scheduled replacement publish.

The wrapper centralizes the root/control epoch check and active local-store config load so both selected and direct scheduled publish routes share the same relation boundary. The selected route now calls the safe internal implementation after validating inputs instead of forwarding through another unsafe block.

No new safe public raw-relation API was added; the existing exported entry points remain `unsafe`.

## Unsafe Count

- before this slice: `1136`
- after this slice: `1135`

## Validation

Artifacts are packet-local under `reviews/task-50/389-spire-scheduled-publish-boundary/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-update-relation.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1135`.
- `unsafe-ledger-generate.log`: generated 1135 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1135 current unsafe rows.
