# Review Request: SPIRE CustomScan Plan Metadata View

## Scope

This packet reviews commit `c97435f959cd364cc8d178cbcbcad229123292e8` (`Add SPIRE CustomScan plan metadata view`).

The slice adds a lifetime-bound `CustomScanPlan<'a>` view in `src/am/ec_spire/custom_scan/plan_private.rs` and routes provider-owned CustomScan plan metadata through safe methods after one callback-scoped constructor.

## Unsafe Burndown

- Removed raw-plan metadata helpers:
  - `custom_scan_plan`
  - `custom_scan_mode_from_plan`
  - `custom_scan_index_oid_from_plan`
  - `custom_scan_dml_column_list_from_plan`
  - `custom_scan_dml_pk_column_from_plan`
  - `custom_scan_custom_private`
- Converted vector top-k plan-private reads to `custom_scan_top_k_from_plan(CustomScanPlan<'_>)`.
- Converted executor/explain mode, index OID, DML column-list, and DML PK-column reads to safe `CustomScanPlan` methods.
- Made `custom_scan_dml_plan_private_copy_roundtrip_for_test` a safe test helper and removed the no-longer-needed unsafe test wrapper at the call site.

Unsafe ledger movement:

- previous packet 171 ledger: `1866`
- packet 172 ledger: `1858`
- net reduction: `8`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_spire/custom_scan/plan_private.rs`: `25 -> 22`
- `src/am/ec_spire/custom_scan/begin_exec.rs`: `25 -> 22`
- `src/am/ec_spire/custom_scan/dml.rs`: `24 -> 23`

## Validation

Packet-local artifacts are under `reviews/task-50/172-spire-customscan-plan-metadata-view/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

Additional attempted coverage:

- `cargo-pgrx-test-plan-private-copyobject-pg18.log` compiled but the test binary failed before running the selected test with `undefined symbol: BufferBlocks`. This is recorded as an environment/runtime launch failure, not as passing validation.

## Reviewer Focus

Please check that `CustomScanPlan<'a>` does not overstate the lifetime guarantee from PostgreSQL callback-owned `CustomScan` pointers, and that replacing the removed unsafe helpers with safe methods does not create the helper anti-pattern called out in the soundness audit.
