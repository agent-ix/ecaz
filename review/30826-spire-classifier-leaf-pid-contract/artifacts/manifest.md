# Artifact Manifest: 30826 SPIRE Classifier Leaf-Pid Contract

## `cargo-test-classify-centroid-lib.log`

- head SHA: `a103778be422724cff212cb8f514dbe2a7d6208c`
- packet/topic: `30826 / spire-classifier-leaf-pid-contract`
- lane / fixture / storage format / rerank mode: PG18 focused classifier SQL
  fixtures; single-level and recursive routing centroid classification; no
  rerank
- command used:
  `script -q -e -c "cargo test classify_centroid --lib" review/30826-spire-classifier-leaf-pid-contract/artifacts/cargo-test-classify-centroid-lib.log`
- timestamp: 2026-05-11T09:15:01-07:00
- isolated/shared surface: isolated pg_test database catalog and routing
  objects
- key result lines:
  `test tests::pg_test_ec_spire_classify_centroid_sql ... ok`;
  `test tests::pg_test_ec_spire_classify_centroid_recursive_sql ... ok`;
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1622 filtered out`

## `cargo-fmt-check.log`

- head SHA: `a103778be422724cff212cb8f514dbe2a7d6208c`
- packet/topic: `30826 / spire-classifier-leaf-pid-contract`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30826-spire-classifier-leaf-pid-contract/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T09:15:01-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `a103778be422724cff212cb8f514dbe2a7d6208c`
- packet/topic: `30826 / spire-classifier-leaf-pid-contract`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30826-spire-classifier-leaf-pid-contract/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T09:15:01-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `a103778be422724cff212cb8f514dbe2a7d6208c`
- packet/topic: `30826 / spire-classifier-leaf-pid-contract`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30826-spire-classifier-leaf-pid-contract/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T09:15:01-07:00
- isolated/shared surface: staged code changes only
- key result lines: command exited successfully with no whitespace errors
