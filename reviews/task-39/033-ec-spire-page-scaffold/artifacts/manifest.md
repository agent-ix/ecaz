# Artifact manifest

- Code scaffold SHA: `f360f4264bd1dec6e3a0d04b6ff20bb02b4f1320`
- Task bucket: `reviews/task-39/033-ec-spire-page-scaffold`
- Lane: `ec_spire/page.rs` 0% → 11.01% via careful shadow scaffold
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `coverage/summary.txt`, `careful-summary.txt`, `root-summary.txt`

- Command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/033-ec-spire-page-scaffold/artifacts/coverage`
- Timestamp: 2026-05-19
- Key result: `am/ec_spire/page.rs 0.00 → 11.01` line coverage.

### `coverage/coverage.json`, `careful-coverage.json`

- Same coverage run. Raw cargo-llvm-cov JSON exports.

### `changed-files.txt`

- Single path (`src/am/ec_spire/page.rs`) for the delta check.

### `coverage-delta-check.log`

- Command: `scripts/check_coverage_delta.sh
  artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv
  artifacts/changed-files.txt`
- Timestamp: 2026-05-19
- Key result: `coverage ok: am/ec_spire/page.rs actual=11.01
  baseline=11.01`.

### `coverage-baseline-check.log`

- Command: `scripts/check_coverage_baseline_complete.sh`
- Timestamp: 2026-05-19
- Key result: `coverage baseline complete for 40 critical paths`.
