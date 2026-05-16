# Artifact Manifest: 30814 SPIRE CustomScan Tuple Payload Slots

## `cargo-test-customscan.log`

- head SHA: `730b7f79`
- packet/topic: `30814-spire-customscan-tuple-payload-slots`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  fixtures, `ecvector_spire_ip_ops`, default storage/rerank settings
- command used:
  `script -q -c 'cargo test customscan --lib' review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-test-customscan.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: isolated pg_test tables; CustomScan execution still
  uses unresolved remote descriptor gates except for the direct virtual-slot
  payload fixture
- key result lines:
  `test tests::pg_test_ec_spire_customscan_tuple_payload_stores_virtual_slot ... ok`
  `test tests::pg_test_ec_spire_customscan_explain_remote_order_limit ... ok`
  `test tests::pg_test_ec_spire_customscan_exec_accepts_parameter_query - should panic ... ok`
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1608 filtered out`

## `cargo-test-tuple-payload.log`

- head SHA: `730b7f79`
- packet/topic: `30814-spire-customscan-tuple-payload-slots`
- lane / fixture / storage format / rerank mode: PG18 focused tuple-payload
  endpoint and executor request fixtures
- command used:
  `script -q -c 'cargo test tuple_payload --lib' review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-test-tuple-payload.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: isolated pg_test tables plus Rust-side executor
  request construction
- key result lines:
  `test am::ec_spire::production_executor_state_tests::production_executor_heap_receive_requests_carry_tuple_payload_columns ... ok`
  `test tests::pg_test_ec_spire_remote_search_tuple_payload_missing_ctid_signal ... ok`
  `test tests::pg_test_ec_spire_remote_search_tuple_payload_side_channel ... ok`
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-fmt-check.log`

- head SHA: `730b7f79`
- packet/topic: `30814-spire-customscan-tuple-payload-slots`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -c 'cargo fmt --check' review/30814-spire-customscan-tuple-payload-slots/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: workspace formatting check
- key result lines:
  command exited successfully; output contains the repository's existing stable
  rustfmt warnings about nightly-only import options

## `git-diff-check.log`

- head SHA: `730b7f79`
- packet/topic: `30814-spire-customscan-tuple-payload-slots`
- lane / fixture / storage format / rerank mode: whitespace check for touched
  files
- command used:
  `script -q -c 'git diff --check HEAD -- Cargo.toml src/am/ec_spire/custom_scan.rs src/am/ec_spire/mod.rs src/am/ec_spire/root/hierarchy_snapshots.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/am/ec_spire/root/types.rs src/am/ec_spire/scan/tests/runtime_state.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md' review/30814-spire-customscan-tuple-payload-slots/artifacts/git-diff-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: touched-file diff against code commit
- key result lines:
  command exited successfully with no whitespace errors
