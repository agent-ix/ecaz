# Review Request: SPIRE CustomScan Tuple Payload Slots

Code slice for Step 2 of the ADR-067 CustomScan pivot. This wires the ADR-068
tuple-payload side-channel into the production executor stream and teaches
`EcSpireDistributedScan` to store remote-origin outputs as virtual tuples.

## Scope

- Adds optional tuple-payload JSON to production heap candidate and scan output
  rows.
- Lets the CustomScan executor declare relation attribute names as the requested
  remote payload columns.
- Threads those requested columns into remote heap receive requests.
- Switches the remote heap receive SQL to
  `ec_spire_remote_search_tuple_payload(...)` when tuple payload columns are
  present, while preserving the existing heap-candidate endpoint for legacy AM
  callers.
- Converts remote payload JSON values through PostgreSQL type input functions
  and stores them in the CustomScan scan slot via `ExecStoreVirtualTuple`.
- Keeps local/materialized heap outputs on the existing
  `table_tuple_fetch_row_version` path.
- Updates the Phase 11 tracker with packet `30814`.

## Validation

- `cargo test customscan --lib`
  - Covers CustomScan status, eligibility, EXPLAIN, production-executor gate,
    parameterized query execution, and virtual tuple payload storage.
- `cargo test tuple_payload --lib`
  - Covers tuple-payload endpoint behavior, missing payload signal, request
    column threading, and virtual slot storage.
- `cargo fmt --check`
- `git diff --check HEAD -- Cargo.toml src/am/ec_spire/custom_scan.rs src/am/ec_spire/mod.rs src/am/ec_spire/root/hierarchy_snapshots.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/am/ec_spire/root/types.rs src/am/ec_spire/scan/tests/runtime_state.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Check the executor boundary: AM callers still use the heap-candidate endpoint,
  while CustomScan callers request tuple payload columns.
- Check the slot materialization path, especially JSON value to type-input
  conversion and null/missing-column behavior.
- Check the remaining boundary: this is not the full end-to-end remote-only
  fixture. The next packet still needs a multi-instance CustomScan read fixture
  proving remote rows return without the materialization catalog/register call.

## Artifacts

- `review/30814-spire-customscan-tuple-payload-slots/artifacts/manifest.md`
- `review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-test-customscan.log`
- `review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-test-tuple-payload.log`
- `review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-fmt-check.log`
- `review/30814-spire-customscan-tuple-payload-slots/artifacts/git-diff-check.log`
