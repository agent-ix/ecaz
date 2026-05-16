# Artifact Manifest: 30815 SPIRE CustomScan Loopback Read

## `cargo-test-customscan-lib.log`

- head SHA: `e656fa76`
- packet/topic: `30815-spire-customscan-loopback-read`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  fixtures, `ecvector_spire_ip_ops`, loopback-remote descriptor, `rabitq`
  storage on the new read fixture
- command used:
  `script -q -e -c "cargo test customscan --lib" review/30815-spire-customscan-loopback-read/artifacts/cargo-test-customscan-lib.log`
- timestamp: 2026-05-11T00:17:26-07:00
- isolated/shared surface: isolated pg_test tables; loopback remote uses the
  same PG18 test server through the production remote descriptor/conninfo path
- key result lines:
  `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`
  `test tests::pg_test_ec_spire_customscan_explain_remote_order_limit ... ok`
  `test tests::pg_test_ec_spire_customscan_exec_accepts_parameter_query - should panic ... ok`
  `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1608 filtered out`

## `cargo-test-tuple-payload-lib.log`

- head SHA: `e656fa76`
- packet/topic: `30815-spire-customscan-loopback-read`
- lane / fixture / storage format / rerank mode: PG18 focused tuple-payload
  endpoint, missing-payload, duplicate-CTID, and CustomScan loopback fixtures
- command used:
  `script -q -e -c "cargo test tuple_payload --lib" review/30815-spire-customscan-loopback-read/artifacts/cargo-test-tuple-payload-lib.log`
- timestamp: 2026-05-11T00:17:26-07:00
- isolated/shared surface: isolated pg_test tables plus Rust-side executor
  request construction
- key result lines:
  `test am::ec_spire::production_executor_state_tests::production_executor_heap_receive_requests_carry_tuple_payload_columns ... ok`
  `test tests::pg_test_ec_spire_remote_search_tuple_payload_missing_ctid_signal ... ok`
  `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-fmt-check.log`

- head SHA: `e656fa76`
- packet/topic: `30815-spire-customscan-loopback-read`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30815-spire-customscan-loopback-read/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T00:17:26-07:00
- isolated/shared surface: workspace formatting check
- key result lines:
  command exited successfully; output contains the repository's existing stable
  rustfmt warnings about nightly-only import options

## `git-diff-check.log`

- head SHA: `e656fa76`
- packet/topic: `30815-spire-customscan-loopback-read`
- lane / fixture / storage format / rerank mode: whitespace check for the
  working diff before packet commit
- command used:
  `script -q -e -c "git diff --check" review/30815-spire-customscan-loopback-read/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T00:17:26-07:00
- isolated/shared surface: working tree diff, with unrelated local WIP left
  unstaged
- key result lines:
  command exited successfully with no whitespace errors
