# Artifact manifest

- Head SHA: `b0e375e6db98bf9173a2787b2605180a015e9703`
- Task bucket: `reviews/task-39/025-storage-guard-coverage`
- Lane: storage guard careful coverage
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `focused-guard-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml --lib guard -- --nocapture`
- Timestamp: 2026-05-19
- Key result: 10 guard tests passed.

### `coverage/summary.txt`

- Command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/025-storage-guard-coverage/artifacts/coverage`
- Timestamp: 2026-05-19
- Key result lines:
  - `storage/buffer_guard.rs`: 100.00% line coverage
  - `storage/lock_guard.rs`: 100.00% line coverage
  - `storage/relation_guard.rs`: 100.00% line coverage
  - `storage/scan_guard.rs`: 95.83% line coverage
  - `storage/slot_guard.rs`: 91.30% line coverage
  - `storage/snapshot_guard.rs`: 82.93% line coverage
  - `storage/spi_guard.rs`: 100.00% line coverage

### `changed-files.txt`

- Command: manual packet-local list of the seven ratcheted guard rows.
- Timestamp: 2026-05-19
- Key result: limits the ratchet to `src/storage/*_guard.rs`.

### `coverage-ratchet.log`

- Command: `scripts/check_coverage_delta.sh --ratchet reviews/task-39/025-storage-guard-coverage/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-39/025-storage-guard-coverage/artifacts/changed-files.txt`
- Timestamp: 2026-05-19
- Key result: seven guard rows passed and `fixtures/quality/coverage-baseline.tsv`
  was ratcheted.

### `coverage-baseline-check.log`

- Command: `make coverage-baseline-check`
- Timestamp: 2026-05-19
- Key result: `coverage baseline complete for 40 critical paths`.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Timestamp: 2026-05-19
- Key result: passed with pre-existing warnings.

### `diff-check.log`

- Command: `git diff --check`
- Timestamp: 2026-05-19
- Key result: passed.
