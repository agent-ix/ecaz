# Task 50 Packet 015 Artifact Manifest

- Head SHA: `f7c4c9b9ec7d4425ce9bbcc8589bae5fe96ec935`
- Task bucket: `reviews/task-50/`
- Packet path: `reviews/task-50/015-hnsw-scan-opaque-accessors/`
- Lane: HNSW scan unsafe structural reduction
- Fixture: local source validation
- Storage format: `ec_hnsw`
- Rerank mode: unchanged by this packet
- Isolated one-index-per-table or shared-table surface: N/A
- Timestamp: `2026-05-20T00:17:49-07:00`

## Artifacts

### `block-count-planning-baseline.log`

- Command:
  `rg -n 'src/am/ec_hnsw/scan.rs' reviews/task-50/001-execution-planning/top-15-coverage-map.md`
- Result:
  planning baseline row records `226` unsafe blocks and a required reduction of
  at least `68`.

### `block-count-after.log`

- Command:
  `make unsafe-block-count PATHS='src/am/ec_hnsw/scan.rs'`
- Result:
  `157 src/am/ec_hnsw/scan.rs`

### `rustfmt-touched-check.log`

- Command:
  `rustfmt --edition 2021 --check src/am/ec_hnsw/scan.rs`
- Result:
  passed with existing stable-rustfmt warnings about unstable config keys.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Result:
  passed with existing warnings.

### `cargo-test-ec-hnsw.log`

- Command:
  `cargo test ec_hnsw --lib --no-default-features --features pg18`
- Result:
  test binary built, then failed to launch outside PostgreSQL:
  `undefined symbol: CacheRegisterRelcacheCallback`

### `cargo-fmt-check.log`

- Command:
  `cargo fmt --all --check`
- Result:
  failed on existing repo-wide formatting drift outside this slice. The touched
  file passed the focused rustfmt check above.

### `cargo-clippy-pg18.log`

- Command:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result:
  failed on the existing repo-wide clippy backlog. The log records
  `COMMAND_EXIT_CODE="101"` and ends with `111 previous errors`.

### `git-diff-check.log`

- Command:
  `git diff --check f7c4c9b9^ f7c4c9b9 -- src/am/ec_hnsw/scan.rs`
- Result:
  passed.
