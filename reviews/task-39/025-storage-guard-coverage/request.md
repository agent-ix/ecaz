# Task 39 storage guard coverage

## Summary

Adds the PostgreSQL RAII guard wrappers to the `hardening/careful` coverage
lane through a careful-only fake `pgrx::pg_sys` surface. The harness imports
the production guard files directly and verifies constructor/drop ownership for
buffers, locks, relations, scans, slots, snapshots, and SPI tuple tables.

## Code under review

- Commit: `b0e375e6db98bf9173a2787b2605180a015e9703`
- Added `hardening/careful/src/pg_guards.rs`
- Updated `hardening/careful/src/lib.rs`
- Ratcheted seven `src/storage/*_guard.rs` rows in
  `fixtures/quality/coverage-baseline.tsv`

## Coverage

From `artifacts/coverage/summary.txt`:

| File | Line coverage |
| --- | ---: |
| `storage/buffer_guard.rs` | 100.00% |
| `storage/lock_guard.rs` | 100.00% |
| `storage/relation_guard.rs` | 100.00% |
| `storage/scan_guard.rs` | 95.83% |
| `storage/slot_guard.rs` | 91.30% |
| `storage/snapshot_guard.rs` | 82.93% |
| `storage/spi_guard.rs` | 100.00% |

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib guard -- --nocapture`
  passed: 10 guard tests.
- `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/025-storage-guard-coverage/artifacts/coverage`
  passed and wrote `artifacts/coverage/summary.txt`.
- `scripts/check_coverage_delta.sh --ratchet ...` passed and ratcheted only the
  seven guard rows.
- `make coverage-baseline-check` passed: 40 critical paths complete.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed with pre-existing warnings.
- `git diff --check` passed.

## Review notes

- The fake `pg_sys` is scoped to the careful hardening crate; production guard
  files are reused by path, not copied.
- The harness serializes only these guard tests because the fake PG counters
  are process-global.
