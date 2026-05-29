# Task 66 packet 004 artifact manifest

- Head SHA: `fe1ec5ec9770988536736c32257a6841854563b6`
- Task bucket: `reviews/task-66/004-pairwise-batch-kernels`
- Timestamp: `2026-05-29T12:22:20-0700`
- Lane: M5 local RaBitQ batch kernel validation
- Fixture/storage/rerank: pure Rust RaBitQ scoring path; no PostgreSQL fixture
- Shared-table surface: not applicable

## Artifacts

### `cargo-test-quant-rabitq.log`

- Command:
  `cargo test --lib --no-default-features --features pg18 quant::rabitq > reviews/task-66/004-pairwise-batch-kernels/artifacts/cargo-test-quant-rabitq.log 2>&1`
- Result:
  `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 1879 filtered out; finished in 0.12s`

### `cargo-check-pg18.log`

- Command:
  `cargo check --no-default-features --features pg18 > reviews/task-66/004-pairwise-batch-kernels/artifacts/cargo-check-pg18.log 2>&1`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.08s`

### `criterion-rabitq-pairwise-batch.log`

- Command:
  `cargo bench --features bench --bench quant_score -- quant/rabitq_score --sample-size 10 --measurement-time 1 > reviews/task-66/004-pairwise-batch-kernels/artifacts/criterion-rabitq-pairwise-batch.log 2>&1`
- Key M5 results:
  - `bits1_batch1000`: `70.664 us` p50, down from packet 001 `85.877 us`
  - `bits8_batch1000`: `112.68 us` p50, down from packet 001 `124.17 us`
  - `bits8c3_batch1000`: `113.00 us` p50, down from packet 001 `123.24 us`
  - `bits8c4_batch1000`: `112.70 us` p50, down from packet 001 `122.47 us`
