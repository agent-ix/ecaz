# Manifest: Task 41 Invariant #2 HNSW build parallel DSM scoped slices

- head SHA: `8848b8dd6f7ac853990e1ba2e01d56c9e1f3742a`
- task bucket and packet path:
  `reviews/task-41/131-hnsw-build-parallel-dsm-scoped-slices/`
- lane / fixture / storage format / rerank mode: source lifetime refactor; no
  SQL fixture, storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:23:38Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### fmt-check.log

- command used:
  `cargo fmt --all --check`
- key result lines:
  - command exited successfully.
  - log contains only stable rustfmt warnings for unsupported nightly-only
    import grouping options.

### cargo-check-pg18.log

- command used:
  `cargo check --no-default-features --features pg18`
- key result lines:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
  - pre-existing warning: unused imports in `src/am/mod.rs`.

### git-diff-check.log

- command used:
  `git diff --check HEAD~1 HEAD`
- key result lines:
  - command exited successfully with no output.

### code-diff-stat.log

- command used:
  `git show --stat --oneline --no-renames HEAD`
- key result lines:
  - `8848b8dd Scope HNSW DSM code slices to callbacks`
  - `src/am/ec_hnsw/build_parallel.rs | 54 ++++++++++++++++++++++++++--------------`

### build-parallel-dsm-slice-inventory.log

- command used:
  `rg -n "'static \\[|concurrent_dsm_(code|source)_for_node|with_concurrent_dsm_(code|source)_for_node|from_raw_parts\\(" src/am/ec_hnsw/build_parallel.rs`
- key result lines:
  - no `&'static [` DSM code/source return remains.
  - `with_concurrent_dsm_code_for_node` and
    `with_concurrent_dsm_source_for_node` own the code/source slice creation.
  - remaining `from_raw_parts` hits are DSM readback/slot helper/message/test
    surfaces rather than ordinary palloc scan-state slices.
