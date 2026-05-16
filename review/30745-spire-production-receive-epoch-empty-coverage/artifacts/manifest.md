# Artifact Manifest: 30745 SPIRE Production Receive Epoch Empty Coverage

Packet: `30745-spire-production-receive-epoch-empty-coverage`
Head SHA: `bf539969c285e2080f711b0ed7cacc4a53bc9a89`
Timestamp: `2026-05-10T05:46:08-07:00`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30745-spire-production-receive-epoch-empty-coverage/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode:
  static formatting / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"`; rustfmt emitted the repository's
  existing stable-toolchain warnings for unstable import grouping options.

## cargo-check-pg18.log

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30745-spire-production-receive-epoch-empty-coverage/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode:
  PG18 static compile / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.12s`

## cargo-pgrx-test-prod-receive.log

- Command:
  `script -q -c "cargo pgrx test pg18 prod_receive" review/30745-spire-production-receive-epoch-empty-coverage/artifacts/cargo-pgrx-test-prod-receive.log`
- Lane / fixture / storage format / rerank mode:
  PG18 pgrx / production compact-candidate receive fixtures / RaBitQ indexes /
  strict receive mode, no heap rerank
- Surface isolation:
  each PG18 fixture creates an isolated table/index or schema-local endpoint
  override; no shared-table surface is used for the new top-k-zero or
  stale-epoch cases.
- Key result lines:
  `test tests::pg_test_ec_spire_prod_receive_stale_epoch ... ok`
  `test tests::pg_test_ec_spire_prod_receive_top_k_zero ... ok`
  `test tests::pg_test_ec_spire_prod_receive_identity_mismatch ... ok`
  `test tests::pg_test_ec_spire_prod_receive_local_cancel_remote_cancel ... ok`
  `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1554 filtered out; finished in 23.76s`

## git-diff-check.log

- Command:
  `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30745-spire-production-receive-epoch-empty-coverage/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode:
  whitespace check / changed code and docs / n/a / n/a
- Surface isolation:
  excludes unrelated local `handoff.md`
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"` and no whitespace errors.
