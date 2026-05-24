# Review Request: SPIRE DML Catalog Relation Boundary

## Summary

This checkpoint addresses the SPIRE DML frontdoor catalog helper cluster from the soundness audit: safe helpers were accepting an open raw `pg_sys::Relation` and hiding relcache/tuple descriptor pointer reads.

Code commit: `eea4add400602f7a0bd9488e6fb15dbc555caf93`

## Scope

- Marked the open-heap catalog helper boundary unsafe:
  - `dml_frontdoor_relation_context_catalog_for_open_heap`
  - `dml_frontdoor_tuple_desc_for_relation`
  - `dml_frontdoor_catalog_index_and_pk`
  - `dml_frontdoor_primary_key_column_from_index`
  - `dml_frontdoor_relation_column_names_from_rel`
  - `dml_frontdoor_index_key_column_names_from_rel`
  - `dml_frontdoor_relation_attr_name_and_form`
- Kept the safe OID-based public entry point by opening the heap relation with `HeapRelationGuard`, then acknowledging the raw-relation boundary once.
- Removed redundant inner unsafe blocks now covered by the helper-level open-relation contract.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: `60` unsafe blocks before, `55` after.
- Current packet-local `src/` unsafe ledger: `1916` rows, checked.

## Completion Audit Note

This advances Wave 2 / SPIRE DML frontdoor relation-handle cleanup. Task 50 is not complete: current ledger output still covers 1916 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
