# Task 50 Packet 211: SPIRE DML Frontdoor Relation Metadata Views

## Summary

This slice consolidates SPIRE DML frontdoor catalog/relcache metadata access behind guard-backed views.

Updated surface:

- `src/am/ec_spire/dml_frontdoor/mod.rs` now uses `DmlFrontdoorHeapRelationView<'_>` for heap relation OID, tuple descriptor, column-name, and attribute lookup.
- The same file now uses `DmlFrontdoorIndexRelationView<'_>` for index relcache form/class metadata.
- The old `dml_frontdoor_tuple_desc_for_relation`, `dml_frontdoor_relation_column_names_from_rel`, and `dml_frontdoor_relation_attr_name_and_form` helper chain was removed.

The remaining unsafe in this frontdoor area is still around PostgreSQL planner/query pointers, callback registration, catalog scans, expression-node inspection, and datum decoding.

## Counts

- `src`: `2561 -> 2559` unsafe references.
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `73 -> 71` unsafe references.
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `20 -> 20` unsafe function contracts.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/dml_frontdoor/mod.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
