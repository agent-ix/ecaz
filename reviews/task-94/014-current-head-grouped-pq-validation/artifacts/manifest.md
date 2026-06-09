# Task 94 Packet 014 Artifact Manifest

- head SHA: `6a0a706a8b9bf06d2f97e44b2ca5e981996160e4`
- task bucket: `reviews/task-94/014-current-head-grouped-pq-validation/`
- timestamp: `2026-06-09T18:18:31Z`
- lane: coder-1 LUT lane, Task 94 grouped-PQ block kernel
- fixture: local Rust unit tests plus matched local PG18 pg_test
- storage format / quant: grouped-PQ / PqFastScan
- isolated/shared table surface: n/a
- AWS/CI usage: none

## Artifacts

### `cargo-test-grouped-pq-lib.log`

- command: `script -q -c "cargo test grouped_pq --lib" reviews/task-94/014-current-head-grouped-pq-validation/artifacts/cargo-test-grouped-pq-lib.log`
- result: passed
- key lines:
  - `running 35 tests`
  - `test tests::pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring ... ok`
  - `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2018 filtered out`
- warning observed: existing `runtime_vector_lanes` dead-code warning in `src/quant/grouped_pq_block/sve.rs` during PG18 feature build.
