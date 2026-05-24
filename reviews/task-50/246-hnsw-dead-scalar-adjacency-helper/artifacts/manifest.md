# Artifact Manifest

- head SHA: `e0deffd520df700321f448e563ba14d4f4278314`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/246-hnsw-dead-scalar-adjacency-helper`
- timestamp: `2026-05-21T14:31:12Z`
- lane: HNSW
- fixture: local compile/static validation
- storage format: HNSW exact/grouped adjacency helpers retained
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-graph.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_hnsw/graph.rs`
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
  - Command: counted `unsafe` lines in `src/am/ec_hnsw/graph.rs` and `src`, then searched for exact `load_graph_adjacency` references.
  - Key lines:
    - `src/am/ec_hnsw/graph.rs unsafe lines: 65`
    - `src unsafe lines: 2443`
    - `deleted symbol references:` with no matches.
