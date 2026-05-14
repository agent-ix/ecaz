# Artifact Manifest: 31023 SPIRE Placement Final Fixture Split

Head SHA: `b5bb0c3086c94429701026fea37924b3bae58dd4`

Packet/topic: `31023-spire-placement-final-fixture-split`

Timestamp: `2026-05-13T18:40:59-07:00`

Lane / fixture / storage format / rerank mode: Phase 12b cleanup fixture
relocation; SPIRE placement fixtures; PostgreSQL/pgrx test storage; rerank
mode not applicable.

Isolation surface: local PG18 pgrx test database; not a measurement run; not
an isolated one-index-per-table or shared-table benchmark surface.

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: pass
- Key lines: rustfmt emitted the repository's stable-channel warnings for
  unstable `imports_granularity` and `group_imports`; command exited 0.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: pass
- Key lines: no output; command exited 0.

### `location-check.log`

- Command: `rg -n 'test_pg18_ec_spire_placement_write_contention_distinct_pk_dml|test_ec_spire_relation_object_tuple_roundtrip' src/tests/mod.rs src/tests/placement.rs`
- Result: pass
- Key lines:
  - `src/tests/placement.rs:407:    fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml()`
  - `src/tests/mod.rs:12373:    fn test_ec_spire_relation_object_tuple_roundtrip()`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/placement.rs src/tests/scan.rs src/tests/remote_search.rs src/tests/vacuum.rs src/tests/insert.rs`
- Result: informational
- Key lines:
  - `34426 src/tests/mod.rs`
  - `570 src/tests/placement.rs`
  - `838 src/tests/scan.rs`
  - `2634 src/tests/remote_search.rs`
  - `1477 src/tests/vacuum.rs`
  - `2814 src/tests/insert.rs`

### `pg18-test-placement-write-contention.log`

- Command: `cargo pgrx test pg18 test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`
- Result: pass
- Key lines:
  - `test tests::pg_test_pg18_ec_spire_placement_write_contention_distinct_pk_dml ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 46.08s`
