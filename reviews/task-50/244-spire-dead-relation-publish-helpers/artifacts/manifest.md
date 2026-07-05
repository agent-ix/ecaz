# Artifact Manifest

- head SHA: `9367d3bbaee9fe7f39e6be98bb33db715279f912`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/244-spire-dead-relation-publish-helpers`
- timestamp: `2026-05-21T14:19:20Z`
- lane: SPIRE
- fixture: local compile/static validation
- storage format: not applicable
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-spire-files.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_spire/build/recursive.rs src/am/ec_spire/update/publish/relation.rs src/am/ec_spire/update/types.rs`
  - Result: passed; stable rustfmt emitted existing unstable-option warnings.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; existing unused SPIRE re-export warning in `src/am/mod.rs`.
- `artifacts/cargo-test-ec-spire-pg18-no-run.log`
  - Command: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; existing Hadamard test-helper dead-code warnings.
- `artifacts/unsafe-counts.log`
  - Command: counted `unsafe` lines in the touched SPIRE files and `src`, then searched for deleted symbol references.
  - Key lines:
    - `touched SPIRE files unsafe lines: 5`
    - `src unsafe lines: 2473`
    - `deleted symbol references:` with no matches.
