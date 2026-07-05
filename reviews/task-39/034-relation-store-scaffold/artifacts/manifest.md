# Artifact manifest

- Code scaffold SHA: `9a818d3360bfd4887adad241e0af6f78924caa44`
- Task bucket: `reviews/task-39/034-relation-store-scaffold`
- Lane: `ec_spire/storage/relation_store.rs` 0% → 3.98% via careful shadow scaffold
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `coverage/summary.txt`, `careful-summary.txt`, `root-summary.txt`

- Command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/034-relation-store-scaffold/artifacts/coverage`
- Timestamp: 2026-05-19
- Key result: `am/ec_spire/storage/relation_store.rs 0.00 → 3.98`
  line coverage.

### `coverage/coverage.json`, `careful-coverage.json`

- Same coverage run. Raw cargo-llvm-cov JSON exports.

### `changed-files.txt`

- Single path (`src/am/ec_spire/storage/relation_store.rs`) for the
  delta check.

### `coverage-delta-check.log`

- Command: `scripts/check_coverage_delta.sh artifacts/coverage/summary.txt
  fixtures/quality/coverage-baseline.tsv artifacts/changed-files.txt`
- Timestamp: 2026-05-19
- Key result: `coverage ok:
  am/ec_spire/storage/relation_store.rs actual=3.98 baseline=3.98`.

### `coverage-baseline-check.log`

- Command: `scripts/check_coverage_baseline_complete.sh`
- Timestamp: 2026-05-19
- Key result: `coverage baseline complete for 40 critical paths`.
