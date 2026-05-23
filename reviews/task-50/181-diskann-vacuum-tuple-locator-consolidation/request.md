# Review Request: DiskANN Vacuum Tuple Locator Consolidation

## Scope

This packet reviews commit `ff6e6c2e962bf46f029620f96f62b11578356a1b` (`Consolidate DiskANN vacuum tuple locator unsafe`).

The slice consolidates the DiskANN vacuum rewrite tuple-location helper so page max-offset reads, item-id lookup, slot validation, tuple bounds validation, and tuple pointer derivation live under one pinned-page unsafe contract.

## Unsafe Burndown

- Keeps `vacuum_page_tuple_location` unsafe because it still accepts a raw PostgreSQL page pointer.
- Consolidates three raw page/item-pointer blocks into one block with the same page-lock/pin invariant.
- Leaves immutable and mutable tuple-slice visitors unchanged.

Unsafe ledger movement:

- previous packet 180 ledger: `1829`
- packet 181 ledger: `1827`
- net reduction: `2`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_diskann/routine.rs`: `56 -> 54`

## Validation

Packet-local artifacts are under `reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that the consolidated tuple locator does not broaden the raw-page contract and that all previous validation ordering is preserved before the tuple pointer is returned.
