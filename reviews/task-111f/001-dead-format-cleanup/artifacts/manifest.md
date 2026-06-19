# Task 111f Dead Format Cleanup Artifact Manifest

- Task bucket: `reviews/task-111f/`
- Packet path: `reviews/task-111f/001-dead-format-cleanup/`
- Code head SHA: `69bdeecf14ee2f8cd256b90168039b8acf5e0b49`
- Timestamp: `2026-06-18T18:58:29-07:00`
- Lane / fixture / storage format / rerank mode: code cleanup validation for IVF dense keeper paths; no benchmark lane.
- Isolated one-index-per-table or shared-table surface: N/A for compile/unit checks; PG18 pgrx fixtures create their own test tables/indexes.

## Artifacts

### `cargo-check-pg18.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111f/001-dead-format-cleanup/artifacts/cargo-check-pg18.log`
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s)`; no errors.

### `cargo-clippy-pg18.log`

- Command: `script -q -e -c "cargo clippy --no-default-features --features pg18 -- -D warnings" reviews/task-111f/001-dead-format-cleanup/artifacts/cargo-clippy-pg18.log`
- Key result: `Finished dev profile [unoptimized + debuginfo] target(s)`; no warnings promoted to errors.

### `cargo-test-ivf-explain-pg18.log`

- Command: `script -q -e -c "cargo test --lib --no-default-features --features pg18 ivf_explain" reviews/task-111f/001-dead-format-cleanup/artifacts/cargo-test-ivf-explain-pg18.log`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2117 filtered out`.

### `cargo-pgrx-test-pg18-ivf-dense.log`

- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_dense" reviews/task-111f/001-dead-format-cleanup/artifacts/cargo-pgrx-test-pg18-ivf-dense.log`
- Key result: `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2113 filtered out`.
- Covered fixtures: dense posting blocks, aligned typed dense posting blocks, dense coalescing disabled, RaBitQ dense blocks, mixed insert rows, and vacuum removal of a build row.

### `cargo-pgrx-test-pg18-coarse-rerank.log`

- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_coarse_rerank_contract_admin_snapshot" reviews/task-111f/001-dead-format-cleanup/artifacts/cargo-pgrx-test-pg18-coarse-rerank.log`
- Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2118 filtered out`.

## Local Checks Not Captured As Logs

- Forbidden source/doc symbol sweep:
  `rg -n "0x26|0x27|0x29|IvfDensePostingPacked|IvfColumnarFrozenList|columnar_page_scatter|dense_posting_pack_pages|columnar_frozen_lists|dense_packed|columnar_frozen|stats_columnar|stats_dense_packed" src docs/on-disk-format.md`
- Key result: no matches.
