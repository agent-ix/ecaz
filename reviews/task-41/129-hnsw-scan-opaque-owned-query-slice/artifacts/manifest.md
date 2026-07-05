# Manifest: Task 41 Invariant #2 HNSW scan opaque-owned query slice

- head SHA: `def32aafdbfba9f80b2447f514b3b558d05ede49`
- task bucket and packet path:
  `reviews/task-41/129-hnsw-scan-opaque-owned-query-slice/`
- lane / fixture / storage format / rerank mode: source lifetime refactor; no
  SQL fixture, storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:19:12Z`
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
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.15s`
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
  - `def32aaf Tie HNSW scan query slice to opaque owner`
  - `src/am/ec_hnsw/scan.rs | 20 +++++++++++---------`

### hnsw-scan-query-slice-inventory.log

- command used:
  `rg -n "scan_query_values\\(|from_raw_parts\\(|query_values\\(" src/am/ec_hnsw/scan.rs`
- key result lines:
  - no `scan_query_values` helper remains.
  - `query_values()` is the owner-tied scan opaque accessor.
  - the remaining non-query `from_raw_parts` hit is a page tuple view for
    Phase D.
