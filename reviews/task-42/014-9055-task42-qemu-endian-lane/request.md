# Review Request: Task 42 Qemu Endian Fixture Lane

## Summary

This checkpoint adds the Task 42 cross-architecture fixture lane:

- `make endian-qemu` cross-compiles and runs `tests/on_disk_fixtures.rs` for `s390x-unknown-linux-gnu` through `qemu-s390x`
- GitHub Actions installs the s390x target, cross linker, and qemu user emulator for the lane
- the qemu lane runs on nightly schedule, manual dispatch, and pushes to `main`
- `docs/on-disk-format.md` documents the lane and moves qemu out of the remaining gaps list

Code checkpoint under review:

- `2e2c77caa2ebc3de4b2e43203472321e9d641ba8` (`Add Task 42 qemu endian fixture lane`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make -n endian-qemu`: command shape validated for the s390x target, linker, qemu runner, and fixture test
- `cargo fmt --all -- --check`: passed
- `make on-disk-fixtures`: 45 passed
- `make upgrade-smoke`: 2 passed
- `make layout-check`: 13 passed

The qemu lane itself is CI-hosted and was not executed locally because the
local host does not have the s390x Rust target, qemu emulator, or s390x
sysroot installed. Validation logs contain the pre-existing `src/am/mod.rs`
unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
- historical live-cluster corpus directories under `fixtures/upgrade/{vN}` when an incompatible format version ships
