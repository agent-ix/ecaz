# Review Request: SPIRE CustomScan DML Boundary Consolidation

## Scope

This packet reviews commit `c63c42635f215d37b3d34042593d392bef4102fc` (`Consolidate SPIRE CustomScan DML unsafe boundaries`).

The slice consolidates adjacent SPIRE CustomScan DML unsafe blocks that shared the same provider-owned `CustomScan` / `CustomScanState` invariant.

## Unsafe Burndown

- Moved expression-list extraction inside the existing `custom_scan_query_from_plan` executor-state boundary.
- Moved tuple-payload relation resolution inside the existing tuple-payload metadata boundary.
- Consolidated DML PK expression extraction/evaluation under one boundary.
- Consolidated DML UPDATE custom expression-list validation and expression extraction under one boundary.

Unsafe ledger movement:

- previous packet 177 ledger: `1848`
- packet 178 ledger: `1843`
- net reduction: `5`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_spire/custom_scan/dml.rs`: `23 -> 18`

## Validation

Packet-local artifacts are under `reviews/task-50/178-spire-customscan-dml-boundary-consolidation/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that each consolidated boundary still has one coherent safety contract and that this did not hide unrelated raw-pointer preconditions inside a broad unsafe block.
