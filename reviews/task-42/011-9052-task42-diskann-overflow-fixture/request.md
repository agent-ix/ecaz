# Review Request: Task 42 DiskANN Overflow Fixture

## Summary

This checkpoint extends Task 42 on-disk fixture coverage for DiskANN duplicate heap-TID overflow tuples:

- overflow tuple fixture and decode check
- byte-swapped heap-TID count rejection check
- public `bench_api` fixture-summary decoder for the otherwise-private overflow tuple
- `docs/on-disk-format.md` fixture inventory update

Code checkpoint under review:

- `289a68f62d9c1b79fd286f8e3a9a8aed6548c8cf` (`Add Task 42 DiskANN overflow fixture`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make on-disk-fixtures`: 45 passed
- `make layout-check`: 13 passed

Both validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- any raw generic page encoding that becomes a durable external byte contract
- broader byte-swapped rejection coverage for bounded multi-byte fields
- qemu cross-arch decode lane, coordinated with Task 48
- `fixtures/upgrade/{vN}` format-version compatibility matrix and upgrade smoke
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
