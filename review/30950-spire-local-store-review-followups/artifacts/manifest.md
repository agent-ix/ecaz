# Artifact Manifest: SPIRE Local Store Review Followups

- Head SHA: `56c3904ad7d4819a7a687cd6eabd2084da58de36`
- Packet/topic: `30950-spire-local-store-review-followups`
- Timestamp: `2026-05-13T01:13:14Z`
- Surface: Phase 12.8 local-store review followups
- Lane / fixture / storage format / rerank mode: PG18;
  `test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`;
  relation-backed two-store local scan/VACUUM fixture; existing fixture rerank
  settings.
- Isolation surface: isolated one-index table fixture; no shared-table remote
  surface.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 56c3904a^ 56c3904a" review/30950-spire-local-store-review-followups/artifacts/git-diff-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Command:
  `script -q -c "cargo fmt --check" review/30950-spire-local-store-review-followups/artifacts/cargo-fmt-check.log`
- Key result lines:
  - `COMMAND_EXIT_CODE="0"`
  - rustfmt emitted the repository's stable-toolchain warnings for nightly-only
    import grouping options.

### `pg18-multistore-sql-vacuum-delta-delete.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores" review/30950-spire-local-store-review-followups/artifacts/pg18-multistore-sql-vacuum-delta-delete.log`
- Key result lines:
  - `Discovered 814 SQL entities: ... 811 functions`
  - `test tests::pg_test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1689 filtered out`
  - `COMMAND_EXIT_CODE="0"`
