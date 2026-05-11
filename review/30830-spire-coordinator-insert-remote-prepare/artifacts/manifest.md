# Artifact Manifest: 30830 SPIRE Coordinator Insert Remote Prepare

## `cargo-test-insert-remote-prepare-lib.log`

- head SHA: `8f5af99ae6cc1280395ea3bedea9101abce42575`
- packet/topic: `30830 / spire-coordinator-insert-remote-prepare`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  insert remote-prepare primitive; loopback remote table, prepared transaction,
  and local placement-directory staging; no rerank
- command used:
  `script -q -e -c "cargo test insert_remote_prepare --lib" review/30830-spire-coordinator-insert-remote-prepare/artifacts/cargo-test-insert-remote-prepare-lib.log`
- timestamp: 2026-05-11T09:55:45-07:00
- isolated/shared surface: isolated pg_test database with loopback libpq
  connection used as the remote; coordinator-local SPIRE index and placement
  directory
- key result lines:
  `test tests::pg_test_ec_spire_insert_remote_prepare_stages_placement_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1633 filtered out`

## `cargo-fmt-check.log`

- head SHA: `8f5af99ae6cc1280395ea3bedea9101abce42575`
- packet/topic: `30830 / spire-coordinator-insert-remote-prepare`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30830-spire-coordinator-insert-remote-prepare/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:56:07-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `8f5af99ae6cc1280395ea3bedea9101abce42575`
- packet/topic: `30830 / spire-coordinator-insert-remote-prepare`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30830-spire-coordinator-insert-remote-prepare/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:56:07-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `8f5af99ae6cc1280395ea3bedea9101abce42575`
- packet/topic: `30830 / spire-coordinator-insert-remote-prepare`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30830-spire-coordinator-insert-remote-prepare/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:57:37-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
