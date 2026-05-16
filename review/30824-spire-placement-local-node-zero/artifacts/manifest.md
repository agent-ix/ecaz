# Artifact Manifest: 30824 SPIRE Placement Local Node Zero

## `cargo-test-ec-spire-placement-lib.log`

- head SHA: `fd7ff5894e1e780542d49f9bb4fbd6bae7b0bc36`
- packet/topic: `30824 / spire-placement-local-node-zero`
- lane / fixture / storage format / rerank mode: PG18 focused placement
  directory catalog SQL fixtures; catalog metadata and direct placement rows;
  no rerank
- command used:
  `script -q -e -c "cargo test ec_spire_placement --lib" review/30824-spire-placement-local-node-zero/artifacts/cargo-test-ec-spire-placement-lib.log`
- timestamp: 2026-05-11T08:55:19-07:00
- isolated/shared surface: isolated pg_test database catalog surface
- key result lines:
  `test tests::pg_test_ec_spire_placement_directory_catalog_sql ... ok`;
  `test tests::pg_test_ec_spire_placement_snapshot_sql ... ok`;
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1618 filtered out`

## `cargo-test-placement-batch-lib.log`

- head SHA: `fd7ff5894e1e780542d49f9bb4fbd6bae7b0bc36`
- packet/topic: `30824 / spire-placement-local-node-zero`
- lane / fixture / storage format / rerank mode: PG18 focused placement batch
  registration fixture; placement-directory batch helper; no rerank
- command used:
  `script -q -e -c "cargo test placement_batch --lib" review/30824-spire-placement-local-node-zero/artifacts/cargo-test-placement-batch-lib.log`
- timestamp: 2026-05-11T08:55:19-07:00
- isolated/shared surface: isolated pg_test database catalog surface
- key result lines:
  `test tests::pg_test_ec_spire_register_placement_batch_sql ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1619 filtered out`

## `cargo-fmt-check.log`

- head SHA: `fd7ff5894e1e780542d49f9bb4fbd6bae7b0bc36`
- packet/topic: `30824 / spire-placement-local-node-zero`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30824-spire-placement-local-node-zero/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T08:55:19-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `fd7ff5894e1e780542d49f9bb4fbd6bae7b0bc36`
- packet/topic: `30824 / spire-placement-local-node-zero`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30824-spire-placement-local-node-zero/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T08:55:19-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `fd7ff5894e1e780542d49f9bb4fbd6bae7b0bc36`
- packet/topic: `30824 / spire-placement-local-node-zero`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30824-spire-placement-local-node-zero/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T08:55:19-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
