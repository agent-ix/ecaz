# Artifact manifest: Task 39 SIMD mutation triage

- Head SHA: `f82589c12dee6e4f113f0a5717222ba1cbb611f5`
- Task bucket: `reviews/task-39/005-simd-mutation-triage`
- Timestamp: `2026-05-18T21:35:23Z`
- Lane: Task 39 coverage and mutation testing
- Fixture: local pure-Rust `ecaz-cli` tests plus `ecaz-careful-hardening` library tests
- Storage format: not applicable; no live index/table storage was created
- Rerank mode: not applicable
- Surface isolation: not applicable; this packet uses pure Rust coverage and mutation harnesses, not shared-table or one-index-per-table PostgreSQL benchmark surfaces

## Artifacts

### `coverage/summary.txt`

- Command:
  `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/005-simd-mutation-triage/artifacts/coverage`
- Key result lines:
  - `quant/simd.rs`: `95.18%` line coverage
  - `TOTAL`: `4.58%`

### `coverage/root-summary.txt`

- Command:
  `cargo llvm-cov report --summary-only`
- Notes:
  root workspace summary after `cargo llvm-cov --no-report -p ecaz-cli`.

### `coverage/careful-summary.txt`

- Command:
  `cargo llvm-cov report --manifest-path hardening/careful/Cargo.toml --summary-only`
- Notes:
  careful harness summary after `cargo llvm-cov --no-report --manifest-path hardening/careful/Cargo.toml --lib`.

### `coverage/coverage.json`

- Command:
  `cargo llvm-cov report --json --output-path reviews/task-39/005-simd-mutation-triage/artifacts/coverage/coverage.json`
- Notes:
  root workspace JSON coverage report.

### `coverage/careful-coverage.json`

- Command:
  `cargo llvm-cov report --manifest-path hardening/careful/Cargo.toml --json --output-path reviews/task-39/005-simd-mutation-triage/artifacts/coverage/careful-coverage.json`
- Notes:
  careful harness JSON coverage report.

### `coverage-run.log`

- Command:
  `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/005-simd-mutation-triage/artifacts/coverage`
- Key result:
  careful harness test result includes `96 passed; 0 failed`.

### `coverage-ratchet.log`

- Command:
  `scripts/check_coverage_delta.sh --ratchet reviews/task-39/005-simd-mutation-triage/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`
- Key result:
  `quant/simd.rs actual=95.18 baseline=94.59`; no further ratchet was required because the improvement was below the 2 percentage point threshold.

### `coverage-delta-check.log`

- Command:
  `scripts/check_coverage_delta.sh reviews/task-39/005-simd-mutation-triage/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`
- Key result:
  all current baseline paths pass.

### `coverage-baseline-check.log`

- Command:
  `make coverage-baseline-check`
- Key result:
  `coverage baseline complete for 40 critical paths`

### `careful-simd-tests.log`

- Command:
  `cargo test --package ecaz-careful-hardening --lib simd -- --test-threads=1`
- Key result:
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out`

### `mutants-list.txt`

- Command:
  `cargo mutants --package ecaz-careful-hardening --file 'hardening/careful/src/../../../src/quant/simd.rs' --list`
- Key result:
  lists 9 generated SIMD mutants.

### `mutants-careful-inplace-run.log`

- Command:
  `cargo mutants --in-place --package ecaz-careful-hardening --file 'hardening/careful/src/../../../src/quant/simd.rs' --output reviews/task-39/005-simd-mutation-triage/artifacts/simd-careful-inplace.mutants`
- Key result:
  `9 mutants tested in 19s: 6 caught, 3 unviable`

### `simd-careful-inplace.mutants/mutants.out/`

- Command:
  same as `mutants-careful-inplace-run.log`
- Key files:
  - `caught.txt`: 6 caught mutants
  - `missed.txt`: empty
  - `timeout.txt`: empty
  - `unviable.txt`: 3 compile-unviable mutants
  - `mutants.json` and `outcomes.json`: raw cargo-mutants records
