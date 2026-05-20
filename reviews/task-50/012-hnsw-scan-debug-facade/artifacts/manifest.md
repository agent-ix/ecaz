# Task 50 Packet 012 Artifact Manifest

- Head SHA: `202f9929f2407f503d3e0b6bd02a182ca1359726`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/012-hnsw-scan-debug-facade/`
- Timestamp: `2026-05-20T06:53:04Z`
- Lane / fixture / storage format / rerank mode: not applicable; debug/test
  facade only.
- Isolated one-index-per-table vs shared-table surface: not applicable.

## Artifacts

### `block-count-before.log`

- Command:
  `printf ' 356 src/am/ec_hnsw/scan_debug.rs\n'`
- Key result:
  `356 src/am/ec_hnsw/scan_debug.rs`

### `block-count-after.log`

- Command:
  `make unsafe-block-count PATHS='src/am/ec_hnsw/scan_debug.rs'`
- Key result:
  `129 src/am/ec_hnsw/scan_debug.rs`

### `rustfmt-touched-check.log`

- Command:
  `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`
- Result:
  passed with existing stable-rustfmt warnings about unstable config keys.

### `cargo-fmt-check.log`

- Command:
  `cargo fmt --all --check`
- Result:
  repo-wide formatting drift reported outside the touched file, including
  `crates/ecaz-cli`, `hardening/careful`, and `src/quant/simd.rs`.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Result:
  passed. Existing warnings remain in `src/am/common/parallel.rs` and
  `src/am/mod.rs`.

### `cargo-clippy-pg18.log`

- Command:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result:
  failed on the existing repo-wide clippy backlog. First failures include
  `src/am/common/parallel.rs`, `src/am/mod.rs`, `src/tests/build.rs`,
  `src/am/ec_diskann/routine.rs`, `src/am/ec_hnsw/shared.rs`, and SPIRE scan /
  update argument-count lints.

### `cargo-test-scan-debug.log`

- Command:
  `cargo test scan_debug --lib --no-default-features --features pg18`
- Result:
  test binary built, then failed to launch outside PostgreSQL:
  `undefined symbol: CacheRegisterRelcacheCallback`.

### `git-diff-check.log`

- Command:
  `git diff --check`
- Result:
  passed.
