# Task 111h Packet 009 Artifact Manifest

- Head SHA: `90d74bd208ff4f847d4cc765e003582eb8765bfa`
- Task bucket: `reviews/task-111h/`
- Packet path: `reviews/task-111h/009-packed-rerank-lifecycle-fixtures/`
- Timestamp: `2026-06-20T05:26:10Z`
- Lane / fixture / storage format / rerank mode: PG18 focused pgrx fixtures;
  `storage_format = 'coarse_rerank'`; `rerank_placement = 'index'`;
  `rerank_format = 'f16'`, `rabitq4`, `rabitq8`; source f32 comparator.
- Surface isolation: isolated one-index-per-table SQL fixtures; no shared-table
  benchmark surface.

## Artifacts

### `cargo-check-pg18.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111h/009-packed-rerank-lifecycle-fixtures/artifacts/cargo-check-pg18.log`
- Result: passed.
- Key lines:
  - `Checking ecaz v0.1.1 (/home/peter/dev/ecaz)`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 7.26s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-pg18-index-placement.log`

- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement" reviews/task-111h/009-packed-rerank-lifecycle-fixtures/artifacts/cargo-pgrx-test-pg18-index-placement.log`
- Result: passed.
- Key lines:
  - `test tests::pg_test_ec_ivf_index_placement_insert_maintains_packed_group ... ok`
  - `test tests::pg_test_ec_ivf_index_placement_vacuum_tombstones_packed_group_slot ... ok`
  - `test tests::pg_test_ec_ivf_index_placement_fewer_rerank_bytes ... ok`
  - `test tests::pg_test_ec_ivf_index_placement_compact_admin_snapshot ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2198 filtered out; finished in 88.44s`
  - `COMMAND_EXIT_CODE="0"`
