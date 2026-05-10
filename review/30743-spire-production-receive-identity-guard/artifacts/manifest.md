# Artifact Manifest: 30743 SPIRE Production Receive Identity Guard

Packet: `30743-spire-production-receive-identity-guard`
Head SHA: `1e97e67ee0f1f30bda5b500abe108a23e8a23ee8`
Timestamp: `2026-05-10T05:21:12-07:00`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30743-spire-production-receive-identity-guard/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode:
  static formatting / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"`; rustfmt emitted the repository's
  existing stable-toolchain warnings for unstable import grouping options.

## cargo-check-pg18.log

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30743-spire-production-receive-identity-guard/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode:
  PG18 static compile / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.17s`

## cargo-test-compact-receive-request-state.log

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 production_executor_compact_receive_requests_use_dispatch_state --lib" review/30743-spire-production-receive-identity-guard/artifacts/cargo-test-compact-receive-request-state.log`
- Lane / fixture / storage format / rerank mode:
  PG18 Rust state test / production receive request state / n/a / n/a
- Surface isolation:
  in-memory executor-state fixture; no shared table/index surface
- Key result lines:
  `test am::ec_spire::production_executor_state_tests::production_executor_compact_receive_requests_use_dispatch_state ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1556 filtered out`

## cargo-pgrx-test-prod-receive.log

- Command:
  `script -q -c "cargo pgrx test pg18 prod_receive" review/30743-spire-production-receive-identity-guard/artifacts/cargo-pgrx-test-prod-receive.log`
- Lane / fixture / storage format / rerank mode:
  PG18 pgrx / production compact-candidate receive loopback and fault fixtures /
  RaBitQ indexes / strict receive mode, no heap rerank
- Surface isolation:
  each PG fixture creates its own table/index or schema-local endpoint function;
  the transport-only portions use no index surface
- Key result lines:
  `test tests::pg_test_ec_spire_prod_receive_identity_mismatch ... ok`
  `test tests::pg_test_ec_spire_prod_receive_backend_terminated ... ok`
  `test tests::pg_test_ec_spire_prod_receive_remote_query_cancelled ... ok`
  `test tests::pg_test_ec_spire_prod_receive_isolates_node_failures ... ok`
  `test tests::pg_test_ec_spire_prod_receive_local_cancel_remote_cancel ... ok`
  `test tests::pg_test_ec_spire_prod_receive_remote_stmt_timeout ... ok`
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1551 filtered out`

## git-diff-check.log

- Command:
  `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md" review/30743-spire-production-receive-identity-guard/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode:
  whitespace check / changed code and task file / n/a / n/a
- Surface isolation:
  excludes unrelated local `handoff.md`
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"` and no whitespace errors.

