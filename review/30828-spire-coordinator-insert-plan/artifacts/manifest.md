# Artifact Manifest: 30828 SPIRE Coordinator Insert Planning Primitive

## `cargo-test-plan-coordinator-insert-lib.log`

- head SHA: `4e73abfe748283d02467e5677c0e8333468286e2`
- packet/topic: `30828 / spire-coordinator-insert-plan`
- lane / fixture / storage format / rerank mode: PG18 focused coordinator
  insert planning primitive; active SPIRE classifier plus placement tuple
  preparation; no rerank
- command used:
  `script -q -e -c "cargo test plan_coordinator_insert --lib" review/30828-spire-coordinator-insert-plan/artifacts/cargo-test-plan-coordinator-insert-lib.log`
- timestamp: 2026-05-11T09:28:14-07:00
- isolated/shared surface: isolated pg_test database with one local SPIRE index
  and rewritten placement node
- key result lines:
  `test tests::pg_test_ec_spire_plan_coordinator_insert_sql ... ok`;
  `test tests::pg_test_ec_spire_plan_coordinator_insert_rejects_empty_pk_sql - should panic ... ok`;
  `test tests::pg_test_ec_spire_plan_coordinator_insert_rejects_bad_identity_sql - should panic ... ok`;
  `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1626 filtered out`

## `cargo-fmt-check.log`

- head SHA: `4e73abfe748283d02467e5677c0e8333468286e2`
- packet/topic: `30828 / spire-coordinator-insert-plan`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30828-spire-coordinator-insert-plan/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:28:14-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `4e73abfe748283d02467e5677c0e8333468286e2`
- packet/topic: `30828 / spire-coordinator-insert-plan`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30828-spire-coordinator-insert-plan/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:28:14-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `4e73abfe748283d02467e5677c0e8333468286e2`
- packet/topic: `30828 / spire-coordinator-insert-plan`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30828-spire-coordinator-insert-plan/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:28:14-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
