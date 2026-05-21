# Task 50 Packet 257 Artifact Manifest

- head SHA at validation start: `137dd446d40e229c2458991270f24c1e5b503cc7`
- task bucket: `reviews/task-50/257-spire-debug-safe-test-api`
- lane: unsafe burndown, SPIRE debug/test API surface
- fixture: static validation and no-run compile coverage
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable

## Artifacts

### `unsafe-counts.log`

- command: packet-local shell count of broad `src` unsafe grep hits and touched
  file direct `unsafe {` blocks before/after the slice
- timestamp: 2026-05-21
- key result lines:
  - `before_broad_unsafe_grep_hits: 2402`
  - `after_broad_unsafe_grep_hits: 2283`
  - `before_touched_direct_unsafe_blocks: 176`
  - `after_touched_direct_unsafe_blocks: 70`
  - `changed_files: 26`

### `rustfmt-check.log`

- command: `rustfmt --edition 2021 --check src/am/ec_spire/coordinator/debug.rs src/am/ec_spire/vacuum/mod.rs`
- timestamp: 2026-05-21
- result: passed; emitted the repository's stable-toolchain warnings for
  nightly-only rustfmt settings

### `git-diff-check.log`

- command: `git diff --check`
- timestamp: 2026-05-21
- result: passed

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: 2026-05-21
- result: passed
- known warning: existing unused SPIRE DML test re-exports in `src/am/mod.rs`

### `cargo-test-lib-pg18-pg-test-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- timestamp: 2026-05-21
- result: passed
- known warning: existing Hadamard test-only helper dead-code warnings
