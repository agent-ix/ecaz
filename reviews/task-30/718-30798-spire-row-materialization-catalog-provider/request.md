# Review Request: SPIRE Row Materialization Catalog Provider

## Summary

This packet lands the catalog-backed Stage D remote row materialization
provider for Task 30 Phase 11.5.

The prior provider seam intentionally blocked remote-origin AM rows unless a
pre-existing coordinator heap mapping could validate. This change adds that
durable mapping surface:

- ADR-065 records the catalog storage decision requested by reviewer feedback.
- `ec_spire_remote_row_materialization` stores ready coordinator heap mappings
  keyed by index, requested/served epoch, origin node, global vector identity,
  opaque origin row locator, and scan heap relation.
- `ec_spire_register_remote_row_materialization(...)` registers a mapping, and
  `ec_spire_remote_row_materialization_catalog(index_oid)` exposes sanitized
  catalog state.
- The AM result stream batch-loads candidate mappings from the catalog for the
  current scan and validates the materialized coordinator heap TID under the
  executor scan snapshot before setting `xs_heaptid`.
- Remote catalog orphan/index/drop cleanup removes materialization rows with
  the rest of the remote catalog state.

The AM path still performs no scan-time writes. Rows without a ready validated
mapping remain blocked as `remote_row_materialization`; rows with stale or
invisible coordinator heap mappings fail closed before delivery.

## Files

- `ecaz--0.1.0--0.1.1.sql`
- `sql/bootstrap.sql`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/scan/callbacks.rs`
- `src/lib.rs`
- `spec/adr/ADR-065-spire-remote-row-materialization-catalog.md`
- `spec/adr/index.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are under `artifacts/` and described in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `git diff --check`
- `cargo test production_scan_row_materialization --no-default-features --features pg18`
- `cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_remote_row_materialization_catalog_register`
- `cargo pgrx test pg18 test_ec_spire_remote_catalog`

## Reviewer Focus

- Confirm ADR-065 is enough durable rationale for the catalog-backed mapping
  design before broader operator-owned mirror lifecycle work depends on it.
- Confirm the AM path remains read-only with respect to materialization state
  and does not introduce scan-time writes.
- Confirm catalog lookup is sufficiently batched for the current result stream
  and does not perform per-row SPI calls.
- Confirm cleanup semantics cover orphan, index, and drop lifecycle for the new
  table.
- Confirm fail-closed behavior is correct when a mapping row exists but the
  coordinator heap row is not visible under the scan snapshot.
