# Review Request: SPIRE DML Catalog Relation Context

## Scope

This packet adds a non-SPI relation-context loader for the ADR-069 DML
front-door path. The existing `ec_spire_dml_frontdoor_relation_context(...)`
surface remains in place; this packet adds a catalog/relcache-backed sibling
that later planner-hook code can consume without recursive SPI catalog lookup.

Code commit: `88cba0d818bab0e5876c2137983fff95c8a0ce2d`

Changes:

- Adds `dml_frontdoor_relation_context_catalog_row(...)`.
- Adds SQL wrapper `ec_spire_dml_frontdoor_relation_context_catalog(...)`.
- Loads relation context from PostgreSQL relcache/index metadata:
  - opens the heap with `table_open`;
  - enumerates indexes with `RelationGetIndexList`;
  - detects the single allowed `ec_spire` index through `get_index_am_oid`;
  - finds the v1 single-column bigint primary key from index metadata;
  - derives ordinary heap columns from the tuple descriptor;
  - derives indexed embedding columns from `indkey`/`indnkeyatts`, preserving
    INCLUDE-column exclusion.
- Keeps the same v1 fail-closed multi-`ec_spire` error text as the SPI-backed
  context loader.
- Extends PG18 coverage to compare the catalog context against the existing
  SPI-backed context for status, PK column, and embedding columns.

## Validation

- `cargo test dml_frontdoor --lib`
  - 15 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the relcache/index metadata path is safe enough as the planner-hook
   metadata source for the next slice.
2. Confirm the v1 index selection semantics still match the SPI-backed loader:
   at most one `ec_spire` index per heap, otherwise fail closed.
3. Confirm the tuple-descriptor/`indnkeyatts` logic preserves ordinary-column
   and embedding-column parity, including excluding INCLUDE columns.
