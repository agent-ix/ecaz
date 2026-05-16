# Artifact Manifest: 30833 SPIRE Coordinator Insert SQL Helper

## `cargo-test-prepare-coordinator-insert-tuple-payload-lib.log`

- head SHA: `b214ae14b738974eebec6490f7abcf8dfb694e67`
- packet/topic: `30833 / spire-coordinator-insert-sql-helper`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  INSERT composition helper; classifier-selected remote node, typed remote
  tuple-payload endpoint, remote prepared transaction, and local placement
  staging; no rerank
- command used:
  `script -q -e -c "cargo test prepare_coordinator_insert_tuple_payload --lib" review/30833-spire-coordinator-insert-sql-helper/artifacts/cargo-test-prepare-coordinator-insert-tuple-payload-lib.log`
- timestamp: 2026-05-11T10:22:39-07:00
- isolated/shared surface: isolated pg_test database with loopback libpq
  connection used as the remote; coordinator-local SPIRE index, remote SPIRE
  index, and placement directory
- key result lines:
  `test tests::pg_test_ec_spire_prepare_coordinator_insert_tuple_payload_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1636 filtered out`

## `cargo-fmt-check.log`

- head SHA: `b214ae14b738974eebec6490f7abcf8dfb694e67`
- packet/topic: `30833 / spire-coordinator-insert-sql-helper`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30833-spire-coordinator-insert-sql-helper/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T10:22:47-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `b214ae14b738974eebec6490f7abcf8dfb694e67`
- packet/topic: `30833 / spire-coordinator-insert-sql-helper`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30833-spire-coordinator-insert-sql-helper/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T10:22:47-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `b214ae14b738974eebec6490f7abcf8dfb694e67`
- packet/topic: `30833 / spire-coordinator-insert-sql-helper`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30833-spire-coordinator-insert-sql-helper/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T10:23:13-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
