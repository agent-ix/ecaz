# Artifact Manifest: 30832 SPIRE Coordinator Insert Payload Prepare

## `cargo-test-insert-remote-prepare-tuple-payload-lib.log`

- head SHA: `03853af2e480063a690caf7ca972bd4d2c418c4a`
- packet/topic: `30832 / spire-coordinator-insert-payload-prepare`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  insert remote-prepare composition; descriptor remote index regclass, typed
  tuple-payload endpoint, remote prepared transaction, and local placement
  staging; no rerank
- command used:
  `script -q -e -c "cargo test insert_remote_prepare_tuple_payload --lib" review/30832-spire-coordinator-insert-payload-prepare/artifacts/cargo-test-insert-remote-prepare-tuple-payload-lib.log`
- timestamp: 2026-05-11T10:12:50-07:00
- isolated/shared surface: isolated pg_test database with loopback libpq
  connection used as the remote; coordinator-local SPIRE index, remote SPIRE
  index, and placement directory
- key result lines:
  `test tests::pg_test_ec_spire_insert_remote_prepare_tuple_payload_endpoint_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1635 filtered out`

## `cargo-fmt-check.log`

- head SHA: `03853af2e480063a690caf7ca972bd4d2c418c4a`
- packet/topic: `30832 / spire-coordinator-insert-payload-prepare`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30832-spire-coordinator-insert-payload-prepare/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T10:12:56-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `03853af2e480063a690caf7ca972bd4d2c418c4a`
- packet/topic: `30832 / spire-coordinator-insert-payload-prepare`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30832-spire-coordinator-insert-payload-prepare/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T10:12:57-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `03853af2e480063a690caf7ca972bd4d2c418c4a`
- packet/topic: `30832 / spire-coordinator-insert-payload-prepare`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30832-spire-coordinator-insert-payload-prepare/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T10:13:18-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
