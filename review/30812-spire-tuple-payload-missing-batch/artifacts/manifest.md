# Artifact Manifest: 30812 SPIRE Tuple Payload Missing Signal and Batch Fetch

## `cargo-test-tuple-payload.log`

- head SHA: `7c1f3ee5`
- packet/topic: `30812-spire-tuple-payload-missing-batch`
- lane / fixture / storage format / rerank mode: PG18 focused tuple-payload
  endpoint fixtures, `ecvector_spire_ip_ops`, default storage/rerank settings
- command used:
  `script -q -c 'cargo test tuple_payload --lib' review/30812-spire-tuple-payload-missing-batch/artifacts/cargo-test-tuple-payload.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: isolated pg_test tables; endpoint side-channel path
  uses the local heap candidate visibility path and batched CTID payload lookup
- key result lines:
  `test tests::pg_test_ec_spire_remote_search_tuple_payload_missing_ctid_signal ... ok`
  `test tests::pg_test_ec_spire_remote_search_tuple_payload_side_channel ... ok`
  `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1609 filtered out`

## `cargo-fmt-check.log`

- head SHA: `7c1f3ee5`
- packet/topic: `30812-spire-tuple-payload-missing-batch`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -c 'cargo fmt --check' review/30812-spire-tuple-payload-missing-batch/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: workspace formatting check
- key result lines:
  command exited successfully; output contains the repository's existing stable
  rustfmt warnings about nightly-only import options

## `git-diff-check.log`

- head SHA: `7c1f3ee5`
- packet/topic: `30812-spire-tuple-payload-missing-batch`
- lane / fixture / storage format / rerank mode: whitespace check for touched
  files
- command used:
  `script -q -c 'git diff --check HEAD -- src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md' review/30812-spire-tuple-payload-missing-batch/artifacts/git-diff-check.log`
- timestamp: 2026-05-10 America/Los_Angeles
- isolated/shared surface: touched-file diff against code commit
- key result lines:
  command exited successfully with no whitespace errors
