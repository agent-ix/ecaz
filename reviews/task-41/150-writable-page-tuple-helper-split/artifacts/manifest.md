# Artifact manifest

- head SHA: `9b7ec742fbce6890f26855f16a51a9ff475fc71c`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/150-writable-page-tuple-helper-split`
- timestamp: `2026-05-18T05:04:13Z`
- lane: Task 41 invariant #2 writable page tuple helper split
- fixture: code-level validation and writable-helper inventory
- storage format: DiskANN and HNSW page tuple rewrite helpers
- rerank mode: not applicable
- isolated one-index-per-table surface: not applicable

## Artifacts

### `code-commit-stat.log`

- command: `git show --stat --oneline 9b7ec742`
- result: recorded code commit `9b7ec742` and changed file stats.

### `writable-helper-inventory.log`

- command: `rg -n 'with_page_tuple_bytes(_mut)?|with_vacuum_page_tuple_bytes(_mut)?|with_writable_page_tuple_bytes|FnOnce\(\*mut u8, \&\[u8\]\)|\|tuple_ptr, tuple_bytes\|' src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs src/am/ec_hnsw/insert.rs src/am/ec_hnsw/vacuum.rs src/am/ec_hnsw/shared.rs`
- result: helper callbacks now expose read-only `&[u8]` or writable
  `&mut [u8]`; no `FnOnce(*mut u8, &[u8])` or `|tuple_ptr, tuple_bytes|`
  hits remain in the touched helper set.

### `cargo-fmt-check.log`

- command: `cargo fmt --all --check`
- result: passed
- key lines: stable rustfmt emitted the repository's existing warnings that
  `imports_granularity` and `group_imports` require nightly.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- result: passed
- key lines: `Finished dev profile` after the known pre-existing unused
  imports warning in `src/am/mod.rs`.

### `git-diff-check-code-commit.log`

- command: `git diff --check 9b7ec742^ 9b7ec742`
- result: passed

### `git-status.log`

- command: `git status --short --branch`
- result: recorded packet worktree state after code commit.
