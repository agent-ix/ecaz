# Task 50 Packet 207: SPIRE DML Catalog Heap Guard

## Summary

This slice threads `HeapRelationGuard` through the SPIRE DML front-door catalog relation-context helpers. The heap relation is opened once by `HeapRelationGuard::try_access_share`, and downstream catalog helpers now receive the guard instead of accepting bare `pg_sys::Relation` pointers with broad `unsafe fn` contracts.

Converted helper contracts include:

- `dml_frontdoor_tuple_desc_for_relation`
- `dml_frontdoor_relation_context_catalog_for_open_heap`
- `dml_frontdoor_catalog_index_and_pk`
- `dml_frontdoor_primary_key_column_from_index`
- `dml_frontdoor_relation_column_names_from_rel`
- `dml_frontdoor_index_key_column_names_from_rel`
- `dml_frontdoor_relation_attr_name_and_form`

This is a structural cleanup: the remaining local unsafe blocks are the specific relcache/tuple descriptor FFI reads, while relation liveness is now represented by the guard type.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs` unsafe function contracts: `27 -> 20`.
- Raw `src` unsafe-reference count after this slice: `2575`.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/dml_frontdoor/mod.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
