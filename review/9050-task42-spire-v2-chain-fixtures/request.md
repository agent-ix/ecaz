# Review Request: Task 42 SPIRE V2 Chain Fixtures

## Summary

This checkpoint extends Task 42 on-disk fixture coverage for SPIRE V2 partition object layouts:

- leaf V2 partition-object meta fixture, decode check, and byte-swapped version rejection
- leaf V2 partition-object segment fixture and decode check against the meta fixture
- generic partition-object V2 chain meta fixture, decode check, and byte-swapped version rejection
- generic partition-object V2 chain segment fixture and decode check against the chain meta fixture
- public `bench_api` fixture-summary decoders for these internal SPIRE V2 layouts
- `docs/on-disk-format.md` fixture inventory update

Code checkpoint under review:

- `5c48a976f54a1af1b2eba5f301a9332e5e95ea36` (`Add Task 42 SPIRE V2 chain fixtures`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make on-disk-fixtures`: 40 passed
- `make layout-check`: 13 passed

Both validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- remaining HNSW/DiskANN/IVF page-kind fixtures
- broader byte-swapped rejection coverage for bounded multi-byte fields
- qemu cross-arch decode lane, coordinated with Task 48
- `fixtures/upgrade/{vN}` format-version compatibility matrix and upgrade smoke
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
