# Task 94 Packet 015 Artifact Manifest

- head SHA: `5289ae91dac8ae5c99fdb20237b5a89dedea6a3b`
- task bucket: `reviews/task-94/015-sve-vector-lane-warning-cleanup/`
- timestamp: `2026-06-09T18:23:31Z`
- lane: coder-1 LUT lane, Task 94 grouped-PQ block kernel
- fixture: local rustfmt plus local grouped-PQ unit/PG18 pg_test validation
- storage format / quant: grouped-PQ / PqFastScan
- isolated/shared table surface: n/a
- AWS/CI usage: none

## Artifacts

### `cargo-fmt-check.log`

- command: `script -q -c "cargo fmt --check" reviews/task-94/015-sve-vector-lane-warning-cleanup/artifacts/cargo-fmt-check.log`
- result: passed
- key lines: stable rustfmt emitted existing nightly-only config warnings for `imports_granularity` and `group_imports`; command exited 0.

### `cargo-test-grouped-pq-lib.log`

- command: `script -q -c "cargo test grouped_pq --lib" reviews/task-94/015-sve-vector-lane-warning-cleanup/artifacts/cargo-test-grouped-pq-lib.log`
- result: passed
- key lines:
  - `running 35 tests`
  - `test tests::pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring ... ok`
  - `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2018 filtered out`
- warning cleanup evidence: the PG18 feature build completed without the previous `runtime_vector_lanes` dead-code warning from `src/quant/grouped_pq_block/sve.rs`.
