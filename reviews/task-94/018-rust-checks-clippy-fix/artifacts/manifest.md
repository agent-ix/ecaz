# Task 94 Packet 018 Artifact Manifest

- head SHA: `9c56cd84233adb6e4465a546d43e3221b38cd0ad`
- task bucket: `reviews/task-94/018-rust-checks-clippy-fix/`
- timestamp: `2026-06-09T18:54:43Z`
- lane: coder-1 LUT lane, Task 94 grouped-PQ block kernel
- fixture: local Rust Checks clippy equivalent plus grouped-PQ unit/PG18 pg_test validation
- storage format / quant: grouped-PQ / PqFastScan
- isolated/shared table surface: n/a
- AWS/CI usage: none; existing failed PR check was inspected only

## Artifacts

### `cargo-clippy-pg18-bench.log`

- command: `script -q -c "cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings" reviews/task-94/018-rust-checks-clippy-fix/artifacts/cargo-clippy-pg18-bench.log`
- result: passed
- key line: `Finished dev profile`
- covers: local reproduction of the failed Rust Checks `Lint` command from GitHub Actions.

### `cargo-test-grouped-pq-lib.log`

- command: `script -q -c "cargo test grouped_pq --lib" reviews/task-94/018-rust-checks-clippy-fix/artifacts/cargo-test-grouped-pq-lib.log`
- result: passed
- key lines:
  - `running 35 tests`
  - `test tests::pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring ... ok`
  - `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2018 filtered out`
