# Artifact Manifest: 30829 SPIRE Coordinator Insert Dispatch Plan

## `cargo-test-insert-dispatch-lib.log`

- head SHA: `3933310da0af6d4cb7120a4b41e0765dff5c0a81`
- packet/topic: `30829 / spire-coordinator-insert-dispatch-plan`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  insert dispatch planning primitive; remote descriptor, conninfo secret, and
  epoch-window readiness states; no rerank
- command used:
  `script -q -e -c "cargo test insert_dispatch --lib" review/30829-spire-coordinator-insert-dispatch-plan/artifacts/cargo-test-insert-dispatch-lib.log`
- timestamp: 2026-05-11T09:41:54-07:00
- isolated/shared surface: isolated pg_test databases with local SPIRE indexes
  and coordinator-local remote descriptor rows
- key result lines:
  `test tests::pg_test_ec_spire_insert_dispatch_missing_descriptor_sql ... ok`;
  `test tests::pg_test_ec_spire_insert_dispatch_missing_secret_sql ... ok`;
  `test tests::pg_test_ec_spire_plan_coordinator_insert_dispatch_ready_sql ... ok`;
  `test tests::pg_test_ec_spire_plan_coordinator_insert_dispatch_stale_epoch_sql ... ok`;
  `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1629 filtered out`

## `cargo-fmt-check.log`

- head SHA: `3933310da0af6d4cb7120a4b41e0765dff5c0a81`
- packet/topic: `30829 / spire-coordinator-insert-dispatch-plan`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30829-spire-coordinator-insert-dispatch-plan/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:42:02-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `3933310da0af6d4cb7120a4b41e0765dff5c0a81`
- packet/topic: `30829 / spire-coordinator-insert-dispatch-plan`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30829-spire-coordinator-insert-dispatch-plan/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:42:01-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `3933310da0af6d4cb7120a4b41e0765dff5c0a81`
- packet/topic: `30829 / spire-coordinator-insert-dispatch-plan`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30829-spire-coordinator-insert-dispatch-plan/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:42:23-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
