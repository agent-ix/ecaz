# Review Request: SPIRE Tuple Payload Test Slot Guard

## Scope

This packet reviews commit `b37c3024487aef223665531fe6e9ec8979676c3f` (`Use slot guard for SPIRE tuple payload test helper`).

The slice tightens the SPIRE CustomScan tuple-payload pg_test helper boundary.

## Unsafe Burndown

- Changed `custom_scan_store_tuple_payload_json_for_test` to take `&TupleTableSlotGuard` instead of a raw `*mut TupleTableSlot`.
- Removed the unsafe block from `test_ec_spire_customscan_tuple_payload_stores_virtual_slot`.
- Kept the actual tuple-slot write unsafe inside the production JSON writer; this slice only removes the test-only raw-slot helper API.

Unsafe ledger movement:

- previous packet 175 ledger: `1850`
- packet 176 ledger: `1849`
- net reduction: `1`

High-signal file counts from `make unsafe-block-count`:

- `src/tests/custom_scan.rs`: `14 -> 13`
- `src/am/ec_spire/custom_scan/tuple_payload.rs`: remains `6`, with the test helper no longer exposed as `unsafe fn`.

## Validation

Packet-local artifacts are under `reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that accepting `&TupleTableSlotGuard` is a real narrowing of the test helper contract and that the helper does not make arbitrary raw tuple slots safe.
