# Review Request: Task 42 Qemu Cross-Build Fix

## Summary

This checkpoint follows up on the first pushed qemu lane run:

- `make endian-qemu` now sets target-specific s390x rustflags so the cross build does not inherit host `target-cpu=native`
- the qemu CI job installs PGDG PostgreSQL 18 headers and sets `PGRX_PG_CONFIG_PATH` for pgrx build-script discovery
- the qemu job still runs the same big-endian `tests/on_disk_fixtures.rs` suite through `qemu-s390x`

Code checkpoint under review:

- `3b8e7a1d65c7259cb7b835948c18edbfb957e033` (`Fix Task 42 qemu cross-build setup`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make -n endian-qemu`: command shape includes s390x linker, qemu runner, target-specific rustflags, and fixture test
- `cargo fmt --all -- --check`: passed
- `git diff --check HEAD^ HEAD`: passed

The executable qemu run remains CI-hosted because this local host does not
have the s390x Rust target, qemu emulator, or s390x sysroot installed. The
prior CI run `26003525647` is the failing run this checkpoint addresses.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- confirming the qemu CI lane runs green after this follow-up lands
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
- historical live-cluster corpus directories under `fixtures/upgrade/{vN}` when an incompatible format version ships
