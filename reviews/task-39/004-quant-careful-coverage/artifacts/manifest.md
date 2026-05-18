# Artifact manifest: Task 39 quant/careful coverage

- Head SHA: `0bfd30786b5636258eb8f988185eb5faf1397ca1`
- Task bucket: `reviews/task-39/004-quant-careful-coverage`
- Timestamp: `2026-05-18T20:56:13Z`
- Lane: Task 39 coverage baseline, ratchet, and gate wiring
- Fixture: local pure-Rust `ecaz-cli` tests plus `hardening/careful` library tests
- Storage format: not applicable; no live index/table storage was created
- Rerank mode: not applicable
- Surface isolation: not applicable; this packet uses pure Rust coverage harnesses, not shared-table or one-index-per-table PostgreSQL benchmark surfaces

## Artifacts

### `coverage/summary.txt`

- Command:
  `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/004-quant-careful-coverage/artifacts/coverage`
- Notes:
  merged summary from root `ecaz-cli` coverage and `hardening/careful` coverage. Root-only JSON is `coverage/coverage.json`; careful-only JSON is `coverage/careful-coverage.json`.
- Key result lines:
  - `quant/codebook.rs`: `98.15%` line coverage
  - `quant/grouped_pq.rs`: `94.72%`
  - `quant/hadamard.rs`: `92.70%`
  - `quant/mse.rs`: `100.00%`
  - `quant/prod.rs`: `93.02%`
  - `quant/qjl.rs`: `100.00%`
  - `quant/rabitq.rs`: `81.43%`
  - `quant/rotation.rs`: `98.53%`
  - `quant/simd.rs`: `48.00%`
  - `storage/page.rs`: `76.57%`
  - `TOTAL`: `4.51%`

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
  `cargo llvm-cov report --json --output-path reviews/task-39/004-quant-careful-coverage/artifacts/coverage/coverage.json`
- Notes:
  root workspace JSON coverage report.

### `coverage/careful-coverage.json`

- Command:
  `cargo llvm-cov report --manifest-path hardening/careful/Cargo.toml --json --output-path reviews/task-39/004-quant-careful-coverage/artifacts/coverage/careful-coverage.json`
- Notes:
  careful harness JSON coverage report.

### `coverage-ratchet.log`

- Command:
  `scripts/check_coverage_delta.sh --ratchet reviews/task-39/004-quant-careful-coverage/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`
- Key result:
  `coverage baseline ratcheted: fixtures/quality/coverage-baseline.tsv`

### `coverage-delta-check.log`

- Command:
  `scripts/check_coverage_delta.sh reviews/task-39/004-quant-careful-coverage/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`
- Key result:
  all 40 current baseline paths pass against the raised baseline.

### `coverage-baseline-check.log`

- Command:
  `make coverage-baseline-check`
- Key result:
  `coverage baseline complete for 40 critical paths`

### `careful-lib-tests.log`

- Command:
  `cargo test --manifest-path hardening/careful/Cargo.toml --lib`
- Key result:
  `test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

### `make-n-task39-quality.log`

- Command:
  `make -n coverage coverage-baseline-check mutants mutants-full flake-hunt MUTANTS_MODULE=src/quant/prod.rs MUTANTS_JOBS=2`
- Notes:
  dry-run evidence for the Task 39 quality gate surfaces after this slice.

## Additional validation

- `bash -n scripts/hardening.sh`
- `bash -n scripts/check_coverage_delta.sh`
- `bash -n scripts/check_coverage_baseline_complete.sh`
- `python3 -m py_compile scripts/merge_coverage_summaries.py`
- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"`
