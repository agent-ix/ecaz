# Artifact Manifest: 31024 SPIRE Scan Final Fixture Split

Head SHA: `1312b56f43955c0eb4fbb06cc98c44d83446c018`

Packet/topic: `31024-spire-scan-final-fixture-split`

Timestamp: `2026-05-13T18:53:30-07:00`

Lane / fixture / storage format / rerank mode: Phase 12b cleanup fixture
relocation; SPIRE scan fixtures; PostgreSQL/pgrx test storage; rerank mode
not applicable.

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

- Command: `rg -n 'test_ec_spire_empty_build_scan_no_rows|test_ec_spire_empty_pq_fastscan_build_scan_no_rows|test_ec_spire_flat_recursive_same_candidate|test_ec_spire_access_method_is_registered|test_ec_spire_relation_object_tuple_roundtrip' src/tests/mod.rs src/tests/scan.rs`
- Result: pass
- Key lines:
  - `src/tests/scan.rs:841:    fn test_ec_spire_empty_build_scan_no_rows()`
  - `src/tests/scan.rs:865:    fn test_ec_spire_empty_pq_fastscan_build_scan_no_rows()`
  - `src/tests/scan.rs:888:    fn test_ec_spire_flat_recursive_same_candidate()`
  - `src/tests/mod.rs:2488:    fn test_ec_spire_access_method_is_registered()`
  - `src/tests/mod.rs:12204:    fn test_ec_spire_relation_object_tuple_roundtrip()`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/scan.rs src/tests/remote_search.rs src/tests/placement.rs`
- Result: informational
- Key lines:
  - `34257 src/tests/mod.rs`
  - `1008 src/tests/scan.rs`
  - `2634 src/tests/remote_search.rs`
  - `570 src/tests/placement.rs`

### `pg18-test-empty-build-scan-no-rows.log`

- Command: `cargo pgrx test pg18 test_ec_spire_empty_build_scan_no_rows`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_empty_build_scan_no_rows ... ok`

### `pg18-test-empty-pq-fastscan-build-scan-no-rows.log`

- Command: `cargo pgrx test pg18 test_ec_spire_empty_pq_fastscan_build_scan_no_rows`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_empty_pq_fastscan_build_scan_no_rows ... ok`

### `pg18-test-flat-recursive-same-candidate.log`

- Command: `cargo pgrx test pg18 test_ec_spire_flat_recursive_same_candidate`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_flat_recursive_same_candidate ... ok`
