# Review Request: SPIRE Row Materialization Provider Seam

## Summary

This packet lands the first Stage D remote row materialization provider seam for
Task 30 Phase 11.5.

The change keeps ADR-064 intact: the index AM still does not write proxy rows
during `amrescan` / `amgettuple`. Instead, AM result-stream finalization now has
a provider boundary that can convert a remote-origin output into a
coordinator-visible heap TID only when a pre-existing mapping validates:

- requested epoch
- served epoch
- origin node ID
- vector identity bytes
- opaque origin row locator bytes
- scan heap relation OID
- scan-snapshot visibility

The default provider is intentionally empty, so remote-origin rows remain
blocked with `remote_row_materialization` until the catalog-backed mapping
surface lands.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/scan/tests.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are under `artifacts/` and described in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/am/ec_spire/scan/types.rs src/am/ec_spire/scan/tests.rs src/am/ec_spire/scan/tests/runtime_state.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo test production_scan_row_materialization --no-default-features --features pg18`
- `cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18`

## Reviewer Focus

- Confirm the provider seam is the right Stage D boundary before catalog-backed
  materialized mapping storage lands.
- Confirm `coordinator_materialized_heap` is acceptable as an AM-deliverable
  owner while preserving the origin node ID and opaque row locator in the output
  row.
- Confirm missing mappings remain a normal classified blocker, while malformed
  mappings fail before `xs_heaptid` delivery.
