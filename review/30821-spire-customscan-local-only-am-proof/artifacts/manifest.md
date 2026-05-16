# Artifact Manifest: 30821 SPIRE CustomScan Local-Only AM Proof

## `cargo-test-customscan-local-only-lib.log`

- head SHA: `f6f2258c07afd650b017b3ed71f2760c3b3e2613`
- packet/topic: `30821 / spire-customscan-local-only-am-proof`
- lane / fixture / storage format / rerank mode: PG18 focused CustomScan
  planner regression; local-only `ec_spire` placement directory; default
  storage; no remote descriptor
- command used:
  `script -q -e -c "cargo test customscan_does_not_replace_local_only_index_plan --lib" review/30821-spire-customscan-local-only-am-proof/artifacts/cargo-test-customscan-local-only-lib.log`
- timestamp: 2026-05-11T08:35:19-07:00
- isolated/shared surface: isolated pg_test table and index; no remote
  descriptor or remote placement rewrite
- key result lines:
  `test tests::pg_test_ec_spire_customscan_does_not_replace_local_only_index_plan ... ok`;
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1619 filtered out`

## `cargo-fmt-check.log`

- head SHA: `f6f2258c07afd650b017b3ed71f2760c3b3e2613`
- packet/topic: `30821 / spire-customscan-local-only-am-proof`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30821-spire-customscan-local-only-am-proof/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T08:35:00-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-check.log`

- head SHA: `f6f2258c07afd650b017b3ed71f2760c3b3e2613`
- packet/topic: `30821 / spire-customscan-local-only-am-proof`
- lane / fixture / storage format / rerank mode: working diff whitespace check
- command used:
  `script -q -e -c "git diff --check" review/30821-spire-customscan-local-only-am-proof/artifacts/git-diff-check.log`
- timestamp: 2026-05-11T08:35:00-07:00
- isolated/shared surface: tracked working diff before code commit, with
  unrelated local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors

## `git-diff-cached-check.log`

- head SHA: `f6f2258c07afd650b017b3ed71f2760c3b3e2613`
- packet/topic: `30821 / spire-customscan-local-only-am-proof`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30821-spire-customscan-local-only-am-proof/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T08:36:00-07:00
- isolated/shared surface: staged code/tracker changes only
- key result lines: command exited successfully with no whitespace errors
