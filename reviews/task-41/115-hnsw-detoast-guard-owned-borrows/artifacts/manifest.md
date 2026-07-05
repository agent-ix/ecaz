# Manifest: Task 41 HNSW Detoast Guard-Owned Borrows

- head SHA: `ffe7338d84160b743e751e2c51e64a5b3e650832`
- task bucket and packet path:
  `reviews/task-41/115-hnsw-detoast-guard-owned-borrows/`
- timestamp: `2026-05-18T02:48:21Z`
- lane / fixture / storage format / rerank mode: HNSW source Datum lifetime
  refactor; no SQL fixture, storage-format matrix, or rerank-mode execution.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  is a compile-time/local source-lifetime slice.

## Artifacts

### code-diff-stat.log

- command used: `git show --stat --oneline --no-renames HEAD`
- key result lines:
  - `ffe7338d Keep HNSW detoast borrows guard-owned`
  - `src/am/ec_hnsw/source.rs | 114 ++++++++++++++++++++++++++---------------------`

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
