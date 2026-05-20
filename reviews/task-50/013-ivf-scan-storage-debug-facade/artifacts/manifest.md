# Task 50 Packet 013 Artifact Manifest

- Head SHA: `137ff87f18fe8b5790511c243df6cc4eabd25a76`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/013-ivf-scan-storage-debug-facade/`
- Timestamp: `2026-05-20T07:01:55Z`
- Lane / fixture / storage format / rerank mode: not applicable; no benchmark
  lane claimed.
- Isolated one-index-per-table vs shared-table surface: not applicable.

## Artifacts

### `block-count-before.log`

- Command:
  `printf ' 102 src/am/ec_ivf/scan.rs\n'`
- Key result:
  `102 src/am/ec_ivf/scan.rs`

### `block-count-after.log`

- Command:
  `make unsafe-block-count PATHS='src/am/ec_ivf/scan.rs'`
- Key result:
  `69 src/am/ec_ivf/scan.rs`

### `rustfmt-touched-check.log`

- Command:
  `rustfmt --edition 2021 --check src/am/ec_ivf/scan.rs`
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

### `cargo-test-ec-ivf.log`

- Command:
  `cargo test ec_ivf --lib --no-default-features --features pg18`
- Result:
  test binary built, then failed to launch outside PostgreSQL:
  `undefined symbol: CacheRegisterRelcacheCallback`.

### `git-diff-check.log`

- Command:
  `git diff --check`
- Result:
  passed.
