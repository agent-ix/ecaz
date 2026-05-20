# Artifact manifest

- Head SHA: `fa98762901f69b72995ae5f4ceee604ed5407140` (parent of the
  baseline-update commit; the packet itself raises the baseline TSV
  on top of this SHA).
- Task bucket: `reviews/task-39/032-spire-storage-coverage-raise`
- Lane: SPIRE storage coverage ratchet for slices 028–031
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `coverage/summary.txt`, `careful-summary.txt`, `root-summary.txt`

- Command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/032-spire-storage-coverage-raise/artifacts/coverage`
- Timestamp: 2026-05-19
- Key result: 4 SPIRE storage rows lifted above their pre-028
  baseline — `leaf_v2_parts.rs 77.52 → 81.03`, `local_store.rs
  78.21 → 80.07`, `local_store_set.rs 41.52 → 63.74`, `vec_id.rs
  69.05 → 69.64`.

### `coverage/coverage.json`, `careful-coverage.json`

- Same coverage run.
- Raw cargo-llvm-cov JSON exports — merged root + careful-only.

### `changed-files.txt`

- Packet-local list of 4 SPIRE storage paths covered by 028–031
  test commits, used as the third arg to
  `scripts/check_coverage_delta.sh`.

### `coverage-delta-check.log`

- Command: `scripts/check_coverage_delta.sh
  artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv
  artifacts/changed-files.txt`
- Timestamp: 2026-05-19
- Key result: `coverage ok` for all four checked paths.

### `coverage-baseline-check.log`

- Command: `scripts/check_coverage_baseline_complete.sh`
- Timestamp: 2026-05-19
- Key result: `coverage baseline complete for 40 critical paths`.
