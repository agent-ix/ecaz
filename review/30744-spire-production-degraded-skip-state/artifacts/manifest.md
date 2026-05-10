# Artifact Manifest: 30744 SPIRE Production Degraded Skip State

Packet: `30744-spire-production-degraded-skip-state`
Head SHA: `c2ef2bd7c04bf5baeb5e3948d4dd0fcdab76c8e0`
Timestamp: `2026-05-10T05:34:26-07:00`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30744-spire-production-degraded-skip-state/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode:
  static formatting / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"`; rustfmt emitted the repository's
  existing stable-toolchain warnings for unstable import grouping options.

## cargo-check-pg18.log

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30744-spire-production-degraded-skip-state/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode:
  PG18 static compile / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 4.03s`

## cargo-test-production-executor.log

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 production_executor_ --lib" review/30744-spire-production-degraded-skip-state/artifacts/cargo-test-production-executor.log`
- Lane / fixture / storage format / rerank mode:
  PG18 Rust state tests plus one PG18 dry summary test / production executor
  state fixtures / n/a / n/a
- Surface isolation:
  in-memory executor-state fixtures for degraded behavior; dry SQL summary uses
  one isolated SPIRE index in its PG18 fixture
- Key result lines:
  `test am::ec_spire::production_executor_state_tests::production_executor_degraded_missing_secret_skips_receive_request ... ok`
  `test am::ec_spire::production_executor_state_tests::production_executor_degraded_receive_failure_allows_ready_merge ... ok`
  `test am::ec_spire::production_executor_state_tests::production_executor_degraded_transport_failure_skips_node ... ok`
  `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1539 filtered out`

## cargo-pgrx-test-production-state-summary-dry.log

- Command:
  `script -q -c "cargo pgrx test pg18 production_executor_state_summary_is_dry" review/30744-spire-production-degraded-skip-state/artifacts/cargo-pgrx-test-production-state-summary-dry.log`
- Lane / fixture / storage format / rerank mode:
  PG18 pgrx / production executor state summary dry fixture / RaBitQ index /
  strict summary mode
- Surface isolation:
  one isolated table/index created by the test
- Key result lines:
  `test tests::pg_test_ec_spire_production_executor_state_summary_is_dry ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1559 filtered out`

## git-diff-check.log

- Command:
  `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/types.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30744-spire-production-degraded-skip-state/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode:
  whitespace check / changed code and docs / n/a / n/a
- Surface isolation:
  excludes unrelated local `handoff.md`
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"` and no whitespace errors.

