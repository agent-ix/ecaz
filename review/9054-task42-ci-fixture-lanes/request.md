# Review Request: Task 42 CI Fixture Lanes

## Summary

This checkpoint wires the Task 42 source-tree lanes into normal validation:

- `make ci-quick` now includes `on-disk-fixtures` and `upgrade-smoke`
- GitHub Actions Rust Checks now run:
  - `cargo test --features bench --test on_disk_fixtures`
  - `cargo test --features bench --test upgrade_matrix`
- existing layout assertions remain in the same workflow

Code checkpoint under review:

- `5ecb486ab7a89af5e7b892d740a652aa524be3d5` (`Run Task 42 fixture lanes in CI`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make on-disk-fixtures`: 45 passed
- `make upgrade-smoke`: 2 passed
- `make layout-check`: 13 passed

All validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- qemu cross-arch decode lane, coordinated with Task 48
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
- historical live-cluster corpus directories under `fixtures/upgrade/{vN}` when an incompatible format version ships
