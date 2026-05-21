# Artifact Manifest

- head SHA: `282b7fb6b359d73c7d5a38b86581ee2b4b5fa17d`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/245-hnsw-dead-graph-traversal-wrappers`
- timestamp: `2026-05-21T14:27:10Z`
- lane: HNSW
- fixture: local compile/static validation
- storage format: HNSW graph traversal wrappers; storage-descriptor APIs retained
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-hnsw-files.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_hnsw/graph.rs src/am/ec_hnsw/scan.rs`
  - Result: passed; stable rustfmt emitted existing unstable-option warnings.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; existing unused SPIRE re-export warning in `src/am/mod.rs`.
- `artifacts/cargo-test-ec-hnsw-pg18-no-run.log`
  - Command: `cargo test --lib ec_hnsw --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; existing Hadamard test-helper dead-code warnings.
- `artifacts/unsafe-counts.log`
  - Command: counted `unsafe` lines in the touched HNSW files and `src`, then searched for exact deleted symbol references.
  - Key lines:
    - `src/am/ec_hnsw/graph.rs unsafe lines: 68`
    - `src/am/ec_hnsw/scan.rs unsafe lines: 202`
    - `src unsafe lines: 2446`
    - `deleted symbol references:` with no matches.
