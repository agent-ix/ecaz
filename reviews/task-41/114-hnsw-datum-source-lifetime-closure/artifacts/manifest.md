# Manifest: Task 41 HNSW Datum Source Lifetime Closure

- head SHA: `e5dbe022da79d969757cdccd059e841695aa3493`
- task bucket and packet path:
  `reviews/task-41/114-hnsw-datum-source-lifetime-closure/`
- timestamp: `2026-05-18T02:25:17Z`
- lane / fixture / storage format / rerank mode: source lifetime API refactor;
  HNSW build/insert/vacuum/grouped heap rerank and IVF heap rerank callers; no
  PostgreSQL fixture or storage-format matrix.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  is a compile-time API/lifetime slice, not a SQL benchmark or execution lane.

## Artifacts

### code-diff-stat.log

- command used: `git show --stat --oneline --no-renames HEAD`
- key result lines:
  - `e5dbe022 Constrain HNSW Datum source lifetimes`
  - `6 files changed, 110 insertions(+), 82 deletions(-)`

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
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.14s`
  - warning is the pre-existing unused import group in `src/am/mod.rs`.
