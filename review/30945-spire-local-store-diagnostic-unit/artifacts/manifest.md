# Artifact Manifest: SPIRE Local Store Diagnostic Unit

- Head SHA: `a5322647ba6554f350b7345d7f82da50d4bdfe29`
- Packet/topic: `30945-spire-local-store-diagnostic-unit`
- Timestamp: `2026-05-13T00:13:32Z`
- Surface: local PG18 two-store SPIRE index diagnostics
- Lane / fixture / storage format / rerank mode: PG18;
  `test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`;
  relation-backed two local stores on `pg_default,pg_default`; existing index
  fixture rerank settings unchanged.
- Isolation surface: isolated one-index table fixture with two local store
  relations; no shared-table remote surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check a5322647^ a5322647" review/30945-spire-local-store-diagnostic-unit/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30945-spire-local-store-diagnostic-unit/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
  - rustfmt emitted the repository's stable-toolchain warnings for nightly-only
    import grouping options.

### `pg18-multistore-sql-vacuum-diagnostic-unit.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores" review/30945-spire-local-store-diagnostic-unit/artifacts/pg18-multistore-sql-vacuum-diagnostic-unit.log`
- Key result lines:
  - `test tests::pg_test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1688 filtered out`
  - `COMMAND_EXIT_CODE="0"`
