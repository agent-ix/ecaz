# Artifact manifest

- head SHA: `b24cc194801d71069a8f87363a4dfcac8f6d936a`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/145-hnsw-scan-debug-page-tuple-byte-scope`
- timestamp: `2026-05-18T04:10:13Z`
- lane: Task 41 invariant #2, HNSW scan debug page-tuple byte views
- fixture: code-level validation only
- storage format: HNSW debug collection loops for scalar, TurboQuant V3, and PqFastScan element tags
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

### `git-diff-check-head-scan-debug.log`

- command: `git diff --check HEAD -- src/am/ec_hnsw/scan_debug.rs`
- result: passed
- note: path-scoped because unrelated comparator and benchmark files were dirty
  in the working tree before this slice.
