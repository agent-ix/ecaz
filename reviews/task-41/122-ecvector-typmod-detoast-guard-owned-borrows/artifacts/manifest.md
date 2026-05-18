# Manifest: Task 41 Invariant #2 ecvector typmod detoast guard

- head SHA: `93610edafe311ee8d8ee98f3518590efcd67a581`
- task bucket and packet path:
  `reviews/task-41/122-ecvector-typmod-detoast-guard-owned-borrows/`
- lane / fixture / storage format / rerank mode: source lifetime refactor; no
  SQL fixture, storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:04:09Z`
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
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.17s`
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
  - `93610eda Keep ecvector typmod detoast guard-owned`
  - `src/lib.rs | 46 ++++++++++++++++++++++++++++++++++------------`
