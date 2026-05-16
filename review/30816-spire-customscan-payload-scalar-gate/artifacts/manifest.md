# Artifact Manifest: 30816 SPIRE CustomScan Payload Scalar Gate

## `cargo-test-customscan-lib.log`

- head SHA: `dd854ebd`
- packet/topic: `30816-spire-customscan-payload-scalar-gate`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  fixtures, `ecvector_spire_ip_ops`, default storage/rerank settings except
  existing loopback `rabitq` read fixture
- command used:
  `script -q -e -c "cargo test customscan --lib" review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-test-customscan-lib.log`
- timestamp: 2026-05-11T00:30:16-07:00
- isolated/shared surface: isolated pg_test tables; loopback remote fixture
  uses the same PG18 test server through the production descriptor path
- key result lines:
  `test tests::pg_test_ec_spire_customscan_rejects_array_tuple_payload_projection - should panic ... ok`
  `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`
  `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1608 filtered out`

## `cargo-test-tuple-payload-lib.log`

- head SHA: `dd854ebd`
- packet/topic: `30816-spire-customscan-payload-scalar-gate`
- lane / fixture / storage format / rerank mode: PG18 focused tuple-payload
  endpoint, CustomScan payload, and scalar-gate fixtures
- command used:
  `script -q -e -c "cargo test tuple_payload --lib" review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-test-tuple-payload-lib.log`
- timestamp: 2026-05-11T00:30:16-07:00
- isolated/shared surface: isolated pg_test tables plus Rust-side executor
  request construction
- key result lines:
  `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_executor_stream_tuple_payload_slots ... ok`
  `test tests::pg_test_ec_spire_customscan_rejects_array_tuple_payload_projection - should panic ... ok`
  `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-fmt-check.log`

- head SHA: `dd854ebd`
- packet/topic: `30816-spire-customscan-payload-scalar-gate`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30816-spire-customscan-payload-scalar-gate/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T00:30:16-07:00
- isolated/shared surface: workspace formatting check
- key result lines:
  command exited successfully; output contains the repository's existing stable
  rustfmt warnings about nightly-only import options

## `git-diff-check.log`

- head SHA: `dd854ebd`
- packet/topic: `30816-spire-customscan-payload-scalar-gate`
- lane / fixture / storage format / rerank mode: whitespace check for the
  working diff before packet commit
- command used:
  `script -q -e -c "git diff --check" review/30816-spire-customscan-payload-scalar-gate/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T00:30:16-07:00
- isolated/shared surface: working tree diff, with unrelated local WIP left
  unstaged
- key result lines:
  command exited successfully with no whitespace errors
