# Task 66 packet 002 artifact manifest

- Head SHA at run time: `749a720468eb751db9f2ae5abb75f4251f5b56d8`
- Task bucket: `reviews/task-66/002-prefetch-unsafe-cleanup`
- Timestamp: `2026-05-29T10:52:23-0700`
- Lane: M5 local validation
- Fixture/storage/rerank: pure Rust RaBitQ scoring path; no PostgreSQL fixture
- Shared-table surface: not applicable

## Artifacts

### `cargo-test-quant-rabitq.log`

- Command:
  `cargo test --lib --no-default-features --features pg18 quant::rabitq > reviews/task-66/002-prefetch-unsafe-cleanup/artifacts/cargo-test-quant-rabitq.log 2>&1`
- Result:
  `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 1879 filtered out; finished in 0.13s`

### `cargo-check-pg18.log`

- Command:
  `cargo check --no-default-features --features pg18 > reviews/task-66/002-prefetch-unsafe-cleanup/artifacts/cargo-check-pg18.log 2>&1`
- Result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 18.69s`
