# Manifest: Task 41 Invariant #2 IVF scan opaque-owned slices

- head SHA: `9e1e9d62a67df398a636d55fc402b53781460753`
- task bucket and packet path:
  `reviews/task-41/128-ivf-scan-opaque-owned-slices/`
- lane / fixture / storage format / rerank mode: source lifetime refactor; no
  SQL fixture, storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:16:50Z`
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
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.18s`
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
  - `9e1e9d62 Tie IVF scan slices to opaque owner`
  - `src/am/ec_ivf/scan.rs | 139 ++++++++++++++++++++++++++++----------------------`

### ivf-scan-slice-inventory.log

- command used:
  `rg -n "from_raw_parts\\(|query_values|selected_lists|posting_candidates|next_posting_candidate\\(" src/am/ec_ivf/scan.rs`
- key result lines:
  - `from_raw_parts` calls for query and selected-list scan-state fields are
    inside `EcIvfScanOpaque` methods.
  - `amgettuple` uses `opaque.next_posting_candidate()`.
  - query storage and selected-list storage retain their existing palloc/pfree
    ownership points.
