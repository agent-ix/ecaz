# Artifact Manifest: 30831 SPIRE Remote Insert Tuple Payload

## `cargo-test-remote-insert-tuple-payload-lib.log`

- head SHA: `467777a9057ec73a6dc0f3bfcb4e34a9da9ee29a`
- packet/topic: `30831 / spire-remote-insert-tuple-payload`
- lane / fixture / storage format / rerank mode: PG18 focused remote insert
  tuple-payload endpoint; remote SPIRE index derives heap relation; JSON payload
  inserts `bigint`, `text`, and `ecvector`; no rerank
- command used:
  `script -q -e -c "cargo test remote_insert_tuple_payload --lib" review/30831-spire-remote-insert-tuple-payload/artifacts/cargo-test-remote-insert-tuple-payload-lib.log`
- timestamp: 2026-05-11T10:04:51-07:00
- isolated/shared surface: isolated pg_test database with a local table acting
  as the remote shard endpoint target
- key result lines:
  `test tests::pg_test_ec_spire_remote_insert_tuple_payload_endpoint_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1634 filtered out`

## `cargo-fmt-check.log`

- head SHA: `467777a9057ec73a6dc0f3bfcb4e34a9da9ee29a`
- packet/topic: `30831 / spire-remote-insert-tuple-payload`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30831-spire-remote-insert-tuple-payload/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T10:04:59-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `467777a9057ec73a6dc0f3bfcb4e34a9da9ee29a`
- packet/topic: `30831 / spire-remote-insert-tuple-payload`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30831-spire-remote-insert-tuple-payload/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T10:04:59-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `467777a9057ec73a6dc0f3bfcb4e34a9da9ee29a`
- packet/topic: `30831 / spire-remote-insert-tuple-payload`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30831-spire-remote-insert-tuple-payload/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T10:06:39-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
