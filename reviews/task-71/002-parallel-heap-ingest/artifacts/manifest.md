# Task 71 / Packet 002 Artifact Manifest

- Head SHA: `c5b8b8c06`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/002-parallel-heap-ingest/`
- Slice: IVF parallel heap ingestion
- Storage format: default IVF options exercised by focused tests
- Rerank mode: not applicable
- Surface: isolated pg_test tables; one IVF index per test table
- Timestamp: 2026-06-02 America/Los_Angeles

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `cargo check --no-default-features --features pg18 > reviews/task-71/002-parallel-heap-ingest/artifacts/cargo-check-pg18.log 2>&1`
- Result: passed
- Key lines:
  - `Checking ecaz v0.1.1 (/Users/peter/dev/tqvector)`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 51.31s`

### `cargo-test-ivf-build-parallel.log`

- Command:
  `cargo test --no-default-features --features pg18 am::ec_ivf::build_parallel > reviews/task-71/002-parallel-heap-ingest/artifacts/cargo-test-ivf-build-parallel.log 2>&1`
- Result: passed
- Key lines:
  - `test am::ec_ivf::build_parallel::tests::parallel_build_plan_uses_dedicated_build_coordinator ... ok`
  - `test am::ec_ivf::build_parallel::tests::parallel_build_plan_stays_serial_without_requested_workers ... ok`
  - `test am::ec_ivf::build_parallel::tests::done_message_round_trips ... ok`
  - `test am::ec_ivf::build_parallel::tests::build_tuple_message_round_trips_payload_and_source_bits ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1936 filtered out; finished in 0.00s`

### `cargo-pgrx-test-pg18-ivf-parallel-build.log`

- Command:
  `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts > reviews/task-71/002-parallel-heap-ingest/artifacts/cargo-pgrx-test-pg18-ivf-parallel-build.log 2>&1`
- Result: failed due sandboxed extension install permissions, then rerun escalated
- Key lines:
  - `Operation not permitted (os error 1)`
  - `test tests::pg_test_ec_ivf_parallel_build_workers_and_counts ... FAILED`

### `cargo-pgrx-test-pg18-ivf-parallel-build-escalated.log`

- Command:
  `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts > reviews/task-71/002-parallel-heap-ingest/artifacts/cargo-pgrx-test-pg18-ivf-parallel-build-escalated.log 2>&1`
- Result: passed
- Key lines:
  - `Installing extension`
  - `test tests::pg_test_ec_ivf_parallel_build_workers_and_counts ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1939 filtered out; finished in 32.57s`

### `cargo-check-pg18-shared-counter-validation.log`

- Command:
  `cargo check --no-default-features --features pg18`
- Result: passed, non-escalated
- Key lines:
  - `Checking ecaz v0.1.1 (/Users/peter/dev/tqvector)`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 21.65s`

### `cargo-test-ivf-build-parallel-shared-counter-validation.log`

- Command:
  `cargo test --no-default-features --features pg18 am::ec_ivf::build_parallel`
- Result: passed, non-escalated
- Key lines:
  - `running 4 tests`
  - `test am::ec_ivf::build_parallel::tests::parallel_build_plan_stays_serial_without_requested_workers ... ok`
  - `test am::ec_ivf::build_parallel::tests::parallel_build_plan_uses_dedicated_build_coordinator ... ok`
  - `test am::ec_ivf::build_parallel::tests::done_message_round_trips ... ok`
  - `test am::ec_ivf::build_parallel::tests::build_tuple_message_round_trips_payload_and_source_bits ... ok`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1936 filtered out; finished in 0.00s`
  - Final command exit status: 0 after filtered binary/integration targets
    reported zero matching tests.
