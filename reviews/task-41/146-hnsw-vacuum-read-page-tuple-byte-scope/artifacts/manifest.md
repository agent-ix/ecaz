# Artifact manifest

- head SHA: `d0b6c08a39f83ea39c44b6371270a74381b73c5b`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/146-hnsw-vacuum-read-page-tuple-byte-scope`
- timestamp: `2026-05-18T04:16:49Z`
- lane: Task 41 invariant #2, HNSW vacuum read page-tuple byte views
- fixture: code-level validation only
- storage format: HNSW vacuum read paths for scalar, TurboQuant V3, PqFastScan, neighbor, and rerank tuples
- rerank mode: same-page grouped rerank payload loading plus existing cross-page fallback
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

### `git-diff-check-head-vacuum.log`

- command: `git diff --check HEAD -- src/am/ec_hnsw/vacuum.rs`
- result: passed
- note: path-scoped because unrelated comparator and benchmark files were dirty
  in the working tree before this slice.
