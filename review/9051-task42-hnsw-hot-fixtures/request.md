# Review Request: Task 42 HNSW Hot Tuple Fixtures

## Summary

This checkpoint extends Task 42 on-disk fixture coverage for HNSW hot/cold tuple layouts:

- grouped-hot tuple fixture and decode check
- turbo-hot tuple fixture and decode check
- cold rerank tuple fixture and decode check
- public `bench_api` re-exports for these tuple structs so decode tests can assert the golden bytes directly
- `docs/on-disk-format.md` fixture inventory update

Code checkpoint under review:

- `92bd54b9f60f4b56bbe1465f1ea1a56178463056` (`Add Task 42 HNSW hot tuple fixtures`)

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `make on-disk-fixtures`: 43 passed
- `make layout-check`: 13 passed

Both validation logs contain the pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor exports.

## Remaining Task 42 Gaps

This is not a full Task 42 closeout. Remaining work still includes:

- remaining DiskANN/IVF page-kind fixtures and any raw generic page encoding that becomes a durable external byte contract
- broader byte-swapped rejection coverage for bounded multi-byte fields
- qemu cross-arch decode lane, coordinated with Task 48
- `fixtures/upgrade/{vN}` format-version compatibility matrix and upgrade smoke
- WAL record version tags, coordinated with Task 37
- `pg_upgrade` smoke with ECAZ data present
