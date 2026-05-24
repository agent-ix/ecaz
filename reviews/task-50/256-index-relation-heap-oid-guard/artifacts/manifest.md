# Task 50 Packet 256 Artifact Manifest

- head SHA at validation start: `a4804c2a4942f0fdd265aba2039a0321827b427f`
- task bucket: `reviews/task-50/256-index-relation-heap-oid-guard`
- lane: unsafe burndown, relation guard debug/test heap OID lookup
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
  - `before_broad_unsafe_grep_hits: 2405`
  - `after_broad_unsafe_grep_hits: 2402`
  - `before_touched_direct_unsafe_blocks: 6 + 135 + 37 + 61 = 239`
  - `after_touched_direct_unsafe_blocks: 7 + 134 + 35 + 60 = 236`

### `rustfmt-check.log`

- command: `rustfmt --edition 2021 --check src/storage/relation_guard.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_diskann/routine.rs src/am/ec_hnsw/vacuum.rs`
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
