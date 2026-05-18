# Review Request: Task 42 SPIRE Partition Fixtures

## Summary

This checkpoint extends Task 42 on-disk fixture coverage to SPIRE V1 partition object bodies:

- leaf partition object body fixture, decode check, and byte-swapped version rejection
- root routing partition object body fixture and decode check
- delta partition object body fixture and decode check
- top-graph partition object body fixture and decode check
- public `bench_api` fixture-summary decoders for these internal SPIRE partition object types
- `docs/on-disk-format.md` fixture inventory update

Code checkpoint under review:

- `a96b65669cb289439cb4892eb321e5e99ffd0238` (`Add Task 42 SPIRE partition fixtures`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make on-disk-fixtures`: 34 passed
- `make layout-check`: 13 passed

Both validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- SPIRE leaf V2/chained partition object body fixtures
- remaining HNSW/DiskANN/IVF page-kind fixtures
- broader byte-swapped rejection coverage for bounded multi-byte fields
- qemu cross-arch decode lane, coordinated with Task 48
- `fixtures/upgrade/{vN}` format-version compatibility matrix and upgrade smoke
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
