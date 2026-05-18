# Manifest: Task 41 SPIRE Scan Detoast Guard-Owned Borrows

- head SHA: `8337a6e36efd4eaeb8fad782af2753aa799da65f`
- task bucket and packet path:
  `reviews/task-41/120-spire-scan-detoast-guard-owned-borrows/`
- timestamp: `2026-05-18T02:57:41Z`
- lane / fixture / storage format / rerank mode: SPIRE scan heap-rerank Datum
  lifetime refactor; no SQL fixture, storage-format matrix, or rerank-mode
  execution.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  is a compile-time/local source-lifetime slice.

## Artifacts

### code-diff-stat.log

- command used: `git show --stat --oneline --no-renames HEAD`
- key result lines:
  - `8337a6e3 Keep SPIRE scan detoast borrows guard-owned`
  - `src/am/ec_spire/scan/relation.rs | 48 +++++++++++++++++++++++++++++++---------`

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
