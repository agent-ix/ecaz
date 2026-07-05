# Manifest: Task 41 Invariant #2 HNSW scan debug query slice

- head SHA: `71c8bc25f790e366c3957c76981207b5bc02e7e4`
- task bucket and packet path:
  `reviews/task-41/130-hnsw-scan-debug-opaque-owned-query-slice/`
- lane / fixture / storage format / rerank mode: source lifetime refactor; no
  SQL fixture, storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:21:20Z`
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
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.20s`
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
  - `71c8bc25 Tie HNSW scan debug query slice to opaque owner`
  - `2 files changed, 10 insertions(+), 8 deletions(-)`

### hnsw-scan-debug-query-slice-inventory.log

- command used:
  `rg -n "from_raw_parts\\(|query_values|query_values_or_empty\\(" src/am/ec_hnsw/scan_debug.rs src/am/ec_hnsw/scan.rs`
- key result lines:
  - `scan_debug.rs:167`: debug query copying uses
    `opaque.query_values_or_empty().to_vec()`.
  - `scan.rs` contains the owner methods that create query slices.
  - remaining `scan_debug.rs` `from_raw_parts` hits are page tuple views for
    Phase D.
