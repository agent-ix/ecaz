# Artifact Manifest: 30810 SPIRE CustomScan Executor Stream

## `cargo-test-customscan-exec.log`

- head SHA: `bce09564`
- packet/topic: `30810-spire-customscan-executor-stream`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  execution fixture, `ecvector_spire_ip_ops`, default storage/rerank settings
- command used:
  `script -q -c 'cargo test customscan_exec --lib' review/30810-spire-customscan-executor-stream/artifacts/cargo-test-customscan-exec.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: isolated pg_test table and one rewritten
  remote-placement leaf
- key result lines:
  `test tests::pg_test_ec_spire_customscan_exec_reaches_production_executor - should panic ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-test-custom-scan-status.log`

- head SHA: `bce09564`
- packet/topic: `30810-spire-customscan-executor-stream`
- lane / fixture / storage format / rerank mode: Rust unit plus PG18 status
  SQL surface
- command used:
  `script -q -c 'cargo test custom_scan_status --lib' review/30810-spire-customscan-executor-stream/artifacts/cargo-test-custom-scan-status.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: CustomScan status SQL surface
- key result lines:
  `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_executor_stream_pending_tuple_payload ... ok`
  `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1608 filtered out`

## `cargo-test-customscan-explain.log`

- head SHA: `bce09564`
- packet/topic: `30810-spire-customscan-executor-stream`
- lane / fixture / storage format / rerank mode: PG18 EXPLAIN fixture,
  `ecvector_spire_ip_ops`, default storage/rerank settings
- command used:
  `script -q -c 'cargo test customscan_explain --lib' review/30810-spire-customscan-executor-stream/artifacts/cargo-test-customscan-explain.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: isolated pg_test table with one placement rewritten
  to remote node 2
- key result lines:
  `test tests::pg_test_ec_spire_customscan_explain_remote_order_limit ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-fmt-check.log`

- head SHA: `bce09564`
- packet/topic: `30810-spire-customscan-executor-stream`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -c 'cargo fmt --check' review/30810-spire-customscan-executor-stream/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: workspace formatting check
- key result lines:
  command exited successfully; output contains the repository's existing stable
  rustfmt warnings about nightly-only import options

## `git-diff-check.log`

- head SHA: `bce09564`
- packet/topic: `30810-spire-customscan-executor-stream`
- lane / fixture / storage format / rerank mode: whitespace check for touched
  files
- command used:
  `script -q -c 'git diff --check HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md' review/30810-spire-customscan-executor-stream/artifacts/git-diff-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: touched-file diff against code commit
- key result lines:
  command exited successfully with no whitespace errors
