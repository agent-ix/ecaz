# Artifact manifest

- head SHA: `ee857d12b85d85c1468a50d1497b2351bc24f226`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/143-hnsw-insert-rewrite-page-tuple-byte-view-scope`
- timestamp: `2026-05-18T04:04:39Z`
- lane: Task 41 invariant #2, HNSW insert page-tuple rewrite byte views
- fixture: code-level validation only
- storage format: HNSW insert paths for scalar, TurboQuant V3, and PqFastScan tuples
- rerank mode: not applicable
- isolated one-index-per-table surface: not applicable

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --all --check`
- result: passed
- key lines: stable rustfmt emitted the repository's existing warnings that
  `imports_granularity` and `group_imports` require nightly.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- result: passed
- key lines: `Finished dev profile` after one known pre-existing unused imports
  warning in `src/am/mod.rs`.

### `git-diff-check-head-insert.log`

- command: `git diff --check HEAD -- src/am/ec_hnsw/insert.rs`
- result: passed
- note: path-scoped because unrelated comparator and benchmark files were dirty
  in the working tree before this slice.
