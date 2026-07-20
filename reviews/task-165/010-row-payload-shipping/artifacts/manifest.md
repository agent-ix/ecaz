# Manifest — Task 165 packet 010 (owner-side row-column payload shipping)

- **head SHA:** a4ecd5e7018b2a1f1ad892344ed2b3650fb3d4de
- **task bucket / packet:** reviews/task-165/010-row-payload-shipping
- **branch:** task-165-ec-distann-m3
- **date:** 2026-07-08
- **surface:** pg_test (in-process PG18, one index per table); no committed-DB or
  bench run in this packet — this is a unit-level data-path slice.

## Code under review

- `src/am/ec_distann/remote_endpoint.rs`
  - `resolve_owned_rows` — shared FR-082 epoch + FR-078 ownership + directory
    ctid resolution (factored out of the 009 ctid endpoint).
  - `ec_distann_materialize_row_payloads` (+ `materialize_row_payloads_impl`) —
    owner-side row-column shipping via `typsend`.
  - `build_payload_sql`, `heap_relation_qualified_name`, `quote_ident`,
    `validate_send_function` — SQL construction + injection guards.
- `src/tests/ec_distann_basic.rs`
  - `test_ec_distann_materialize_row_payloads_ships_binary_columns`.

## Commands

- clippy: `cargo clippy --lib --no-default-features --features pg18`
- tests: `cargo pgrx test pg18 --no-default-features --features pg18 distann`

## Key result lines (see test-evidence.log)

- `test ... test_ec_distann_materialize_row_payloads_ships_binary_columns ... ok`
- `test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 2267 filtered out`
- clippy: `Finished` with no warnings.

## Not in this packet (open)

- The coordinator-side CustomScan that consumes `payload_values` and yields
  reconstructed tuples (Slice B.2/B.3).
- The real 3-instance fixture (Slice A) and the 3-worker `ecaz bench suite`
  distinct-recall exit gate (Slice D). These are code/measurement work, not
  bench-provenance evidence for this data-path slice.
