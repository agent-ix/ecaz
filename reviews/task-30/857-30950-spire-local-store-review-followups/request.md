# Review Request: SPIRE Local Store Review Followups

Code checkpoint: `56c3904a` (`Address SPIRE local-store review followups`)

## Scope

- Addresses reviewer P3 followups from packets `30945`, `30946`, and `30947`.
- Changes the PG18 multi-store SQL VACUUM fixture to delete the post-build row,
  so the checked cleanup path now retires a delta-routed row rather than only a
  base-routed row.
- Adds a fixture comment explaining why the test validates through SPIRE
  diagnostics instead of broad heap SELECTs under the fail-closed DML frontdoor.
- Clarifies in diagnostics/design docs that PG18 ReadStream is read-ahead only:
  object decoding, candidate scoring, and heap rerank CPU work still serialize
  in one backend.
- Pins `local_store_execution_mode` as an operator-visible public label and
  cross-references `(node_id, local_store_id)` grouping from the delta reuse
  note.

## Validation

- `git diff --check 56c3904a^ 56c3904a`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and key result lines.

## Review Focus

- Confirm deleting the post-build row is the right way to cover delta-routed
  cleanup in the existing multi-store SQL VACUUM fixture.
- Confirm the documentation wording is strict enough to prevent interpreting
  ReadStream as current multi-store parallel execution.
