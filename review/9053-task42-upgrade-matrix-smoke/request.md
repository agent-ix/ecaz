# Review Request: Task 42 Upgrade Matrix Smoke

## Summary

This checkpoint adds the first Task 42 format-version compatibility matrix smoke:

- `fixtures/upgrade/matrix.csv` records `(AM, format_version, can_read, can_write)` for current supported formats
- `make upgrade-smoke` runs `tests/upgrade_matrix.rs`
- the smoke test enforces unique matrix rows, readable writable formats, existing fixture references, and the current writable set
- `docs/on-disk-format.md` documents the matrix and target

Code checkpoint under review:

- `16d7f08c6bc03590db4e38cae959bc3c94d69f3a` (`Add Task 42 upgrade matrix smoke`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make upgrade-smoke`: 2 passed
- `make on-disk-fixtures`: 45 passed
- `make layout-check`: 13 passed

All validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- historical live-cluster corpus directories under `fixtures/upgrade/{vN}` when an incompatible format version ships
- broader byte-swapped rejection coverage for bounded multi-byte fields
- qemu cross-arch decode lane, coordinated with Task 48
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
