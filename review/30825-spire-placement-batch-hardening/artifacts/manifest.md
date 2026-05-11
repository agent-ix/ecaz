# Artifact Manifest: 30825 SPIRE Placement Batch Hardening

## `cargo-test-placement-batch-lib.log`

- head SHA: `813ff1f289993f7c48b9c9b937aa2f5ceddb95f1`
- packet/topic: `30825 / spire-placement-batch-hardening`
- lane / fixture / storage format / rerank mode: PG18 focused placement batch
  registration SQL fixtures; placement-directory batch helper; no rerank
- command used:
  `script -q -e -c "cargo test placement_batch --lib" review/30825-spire-placement-batch-hardening/artifacts/cargo-test-placement-batch-lib.log`
- timestamp: 2026-05-11T09:07:47-07:00
- isolated/shared surface: isolated pg_test database catalog surface
- key result lines:
  `test tests::pg_test_ec_spire_register_placement_batch_sql ... ok`;
  `test tests::pg_test_ec_spire_register_placement_batch_rejects_null_entry_sql - should panic ... ok`;
  `test tests::pg_test_ec_spire_register_placement_batch_rejects_invalid_sql - should panic ... ok`;
  `test tests::pg_test_ec_spire_register_placement_batch_rejects_duplicate_pk_sql - should panic ... ok`;
  `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1619 filtered out`

## `cargo-fmt-check.log`

- head SHA: `813ff1f289993f7c48b9c9b937aa2f5ceddb95f1`
- packet/topic: `30825 / spire-placement-batch-hardening`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30825-spire-placement-batch-hardening/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:07:47-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `813ff1f289993f7c48b9c9b937aa2f5ceddb95f1`
- packet/topic: `30825 / spire-placement-batch-hardening`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30825-spire-placement-batch-hardening/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:07:47-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `813ff1f289993f7c48b9c9b937aa2f5ceddb95f1`
- packet/topic: `30825 / spire-placement-batch-hardening`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30825-spire-placement-batch-hardening/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:07:47-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
