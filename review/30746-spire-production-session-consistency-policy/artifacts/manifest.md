# Artifact Manifest: 30746 SPIRE Production Session Consistency Policy

Packet: `30746-spire-production-session-consistency-policy`
Head SHA: `8befc3d72c2a58204dccdebed92e783c5f8ce0c3`
Timestamp: `2026-05-10T06:05:13-07:00`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30746-spire-production-session-consistency-policy/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode:
  static formatting / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"`; rustfmt emitted the repository's
  existing stable-toolchain warnings for unstable import grouping options.

## cargo-check-pg18.log

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30746-spire-production-session-consistency-policy/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode:
  PG18 static compile / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.11s`

## cargo-pgrx-test-prod-executor-session-policy.log

- Command:
  `script -q -c "cargo pgrx test pg18 prod_executor_session_policy" review/30746-spire-production-session-consistency-policy/artifacts/cargo-pgrx-test-prod-executor-session-policy.log`
- Lane / fixture / storage format / rerank mode:
  PG18 pgrx / production executor session-policy dry fixture / RaBitQ index /
  no heap rerank
- Surface isolation:
  isolated table/index `ec_spire_prod_session_policy_*`; the fixture first
  reads default strict mode from the session GUC, then rewrites the active
  epoch to degraded before testing a degraded session policy against a remote
  placement.
- Key result lines:
  `test tests::pg_test_ec_spire_prod_executor_session_policy_guc ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1562 filtered out; finished in 24.08s`

## git-diff-check.log

- Command:
  `script -q -c "git diff --check -- src/am/ec_spire/options.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30746-spire-production-session-consistency-policy/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode:
  whitespace check / changed code and docs / n/a / n/a
- Surface isolation:
  excludes unrelated local `handoff.md`
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"` and no whitespace errors.
