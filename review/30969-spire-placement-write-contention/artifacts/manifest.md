# Artifact Manifest: SPIRE Placement Write Contention

- Head SHA: `ff28a1fd8cf7ab6dce05ec070900de1d3ec59102`
- Packet/topic: `30969-spire-placement-write-contention`
- Timestamp: `2026-05-13T05:30:07Z`
- Lane / fixture / storage format / rerank mode: Phase 12.4 PG18
  placement-table write-contention fixture; storage format and rerank mode not
  exercised.
- Surface isolation: shared-table `ec_spire_placement` surface; no
  one-index-per-table isolation.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check ff28a1fd^ ff28a1fd" review/30969-spire-placement-write-contention/artifacts/git-diff-check.log`
- Result lines:
  - Command exited successfully with no diff-check findings.

### `cargo-check-pg18.log`

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30969-spire-placement-write-contention/artifacts/cargo-check-pg18.log`
- Result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.12s`
  - Existing warning: `ecaz` lib has unused imports in `src/am/mod.rs`.

### `cargo-pgrx-test-placement-contention.log`

- Command:
  `script -q -c "cargo pgrx test pg18 test_pg18_ec_spire_placement_write_contention_distinct_pk_dml" review/30969-spire-placement-write-contention/artifacts/cargo-pgrx-test-placement-contention.log`
- Result lines:
  - `test tests::pg_test_pg18_ec_spire_placement_write_contention_distinct_pk_dml ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1696 filtered out; finished in 31.08s`
  - The test would panic if any worker failed, a delete row count was not one,
    placement locks remained waiting, deadlock stats increased, or p99 exceeded
    the predeclared 20s threshold.
