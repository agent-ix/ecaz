# Artifact Manifest: 30835 SPIRE Coordinator Insert Trigger

## `cargo-test-enable-coordinator-insert-trigger-lib.log`

- head SHA: `a56d6fa8b2a472333f4d0ff015fa70991f178636`
- packet/topic: `30835 / spire-coordinator-insert-trigger`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  INSERT trigger test; v1 bigint-PK / `ecvector` / bytea source-identity table
  shape; loopback remote descriptor; remote prepared transaction and local
  placement staging; no rerank
- command used:
  `script -q -e -c "cargo test enable_coordinator_insert_trigger --lib" review/30835-spire-coordinator-insert-trigger/artifacts/cargo-test-enable-coordinator-insert-trigger-lib.log`
- timestamp: 2026-05-11T18:04:28Z
- isolated/shared surface: isolated pg_test database with loopback libpq
  connection used as the remote; coordinator and remote test tables are
  separate relations
- key result lines:
  `test tests::pg_test_ec_spire_enable_coordinator_insert_trigger_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1637 filtered out`

## `cargo-fmt-check.log`

- head SHA: `a56d6fa8b2a472333f4d0ff015fa70991f178636`
- packet/topic: `30835 / spire-coordinator-insert-trigger`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30835-spire-coordinator-insert-trigger/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T18:04:46Z
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `a56d6fa8b2a472333f4d0ff015fa70991f178636`
- packet/topic: `30835 / spire-coordinator-insert-trigger`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30835-spire-coordinator-insert-trigger/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T18:04:46Z
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors
