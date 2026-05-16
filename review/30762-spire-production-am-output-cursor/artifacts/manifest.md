# 30762 Artifact Manifest

Head SHA: `3460dd27e2d8d5520610fde5741380599f6ea2c0`

Packet: `30762-spire-production-am-output-cursor`

Scope: SPIRE Phase 11 Stage D AM cursor integration for production-stream
outputs that are already deliverable as coordinator-local heap TIDs.

Lane: local PG18 compile/static validation. Fixture: AM cursor and Rust
contract helpers only. Storage format: not applicable. Rerank mode: production
stream preserves existing heap-resolution rerank behavior. Surface shape: no
isolated or shared SQL/index fixture was started for this packet.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 13:11 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 13:14 PDT
  - Result: pass, no whitespace errors.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 13:11 PDT
  - Result line: `Finished dev profile ... target(s) in 4.86s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 13:12 PDT
  - Result line: `Finished dev profile ... target(s) in 14.39s`.

- `cargo-test-production-am-output-cursor.log`
  - Command: `script -q -e -c 'cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 13:12 PDT
  - Result: compiled, then failed before running the test body with the known
    direct-test pgrx loader issue, `undefined symbol: SPI_finish`.
