# Manifest: Task 41 HNSW Build Detoast Guard-Owned Borrows

- head SHA: `98380d5c5f97628238bcb2ab33e9c3329cbf2543`
- task bucket and packet path:
  `reviews/task-41/117-hnsw-build-detoast-guard-owned-borrows/`
- timestamp: `2026-05-18T02:51:59Z`
- lane / fixture / storage format / rerank mode: HNSW build tqvector Datum
  lifetime refactor; no SQL fixture, storage-format matrix, or rerank-mode
  execution.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  is a compile-time/local source-lifetime slice.

## Artifacts

### code-diff-stat.log

- command used: `git show --stat --oneline --no-renames HEAD`
- key result lines:
  - `98380d5c Keep HNSW build detoast borrows guard-owned`
  - `src/am/ec_hnsw/build.rs | 46 +++++++++++++++++++++++++++++++++++++---------`

### git-diff-check.log

- command used: `git diff --check HEAD~1 HEAD`
- key result lines:
  - no output; command exited successfully.

### fmt-check.log

- command used: `cargo fmt --all --check`
- key result lines:
  - command exited successfully.
  - log contains existing rustfmt warnings about unstable
    `imports_granularity` / `group_imports` options on stable toolchain.

### cargo-check-pg18.log

- command used: `cargo check --no-default-features --features pg18`
- key result lines:
  - `warning: ecaz (lib) generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`
  - warning is the pre-existing unused import group in `src/am/mod.rs`.
