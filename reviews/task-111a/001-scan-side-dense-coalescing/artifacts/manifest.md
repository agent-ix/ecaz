# Task 111a Packet 001 Artifact Manifest

Task bucket: `reviews/task-111a/001-scan-side-dense-coalescing`
Head SHA: `b47d5b78ccc6a67be3fec2af9da004551e6cb2c6`
Branch: `task-111-ivf-dense-posting-block-layout`
Timestamp: `2026-06-16T22:01:25-07:00`

## Scope

This packet covers Task 111a Phase 1 / Approach A only: scan-side cross-block
dense posting coalescing behind the existing `dense_posting_blocks` gate.

No durable page-format change was made. Approach B, benchmark matrix evidence,
and the final promote / iterate / abandon decision remain pending.

Changed code:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/common/explain.rs`
- `src/tests/ec_ivf.rs`

Benchmark metadata: not a benchmark packet. Lane, fixture, storage format, and
rerank mode are not applicable for these validation artifacts.

## Artifacts

### `cargo-check-lib.log`

- Command: `cargo check -q --lib`
- Capture: `script -q -e -c "cargo check -q --lib" reviews/task-111a/001-scan-side-dense-coalescing/artifacts/cargo-check-lib.log`
- Result: command exited `0`.
- Key lines:
  - `Script done on 2026-06-16 21:55:05-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-check-lib-pg-test.log`

- Command: `cargo check -q --lib --features pg_test`
- Capture: `script -q -e -c "cargo check -q --lib --features pg_test" reviews/task-111a/001-scan-side-dense-coalescing/artifacts/cargo-check-lib-pg-test.log`
- Result: command exited `0`.
- Key lines:
  - `Script done on 2026-06-16 21:55:04-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-test-posting-scratch-soa.log`

- Command: `cargo test -q posting_scratch_soa --lib`
- Capture: `script -q -e -c "cargo test -q posting_scratch_soa --lib" reviews/task-111a/001-scan-side-dense-coalescing/artifacts/cargo-test-posting-scratch-soa.log`
- Result: command exited `0`.
- Key lines:
  - `running 4 tests`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2106 filtered out; finished in 0.00s`

### `cargo-test-ivf-explain-counters.log`

- Command: `cargo test -q ivf_explain_counters --lib`
- Capture: `script -q -e -c "cargo test -q ivf_explain_counters --lib" reviews/task-111a/001-scan-side-dense-coalescing/artifacts/cargo-test-ivf-explain-counters.log`
- Result: command exited `0`.
- Key lines:
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2109 filtered out; finished in 0.00s`

### `cargo-pgrx-test-pg18-dense-posting-blocks.log`

- Command: `cargo pgrx test pg18 dense_posting_blocks`
- Capture: `script -q -e -c "cargo pgrx test pg18 dense_posting_blocks" reviews/task-111a/001-scan-side-dense-coalescing/artifacts/cargo-pgrx-test-pg18-dense-posting-blocks.log`
- Result: command exited `0`.
- PostgreSQL surface: PG18 via `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`.
- Key lines:
  - `running 5 tests`
  - `test tests::pg_test_ec_ivf_dense_posting_blocks_rabitq_scan_build_rows ... ok`
  - `test tests::pg_test_ec_ivf_dense_posting_blocks_scan_mixed_insert_rows ... ok`
  - `test tests::pg_test_ec_ivf_dense_posting_blocks_scan_build_rows ... ok`
  - `test tests::pg_test_ec_ivf_dense_posting_blocks_vacuum_removes_build_row ... ok`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2105 filtered out; finished in 71.59s`
