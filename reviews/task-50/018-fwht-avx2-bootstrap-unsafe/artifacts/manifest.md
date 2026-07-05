---
task: 50
packet: reviews/task-50/018-fwht-avx2-bootstrap-unsafe
head_sha: 9043ecf05b400d755e33f71ffad7ea1eb227aa24
code_commit: 9043ecf0 Consolidate FWHT AVX2 bootstrap unsafe blocks
generated_at: 2026-05-20T01:00:52-07:00
lane: rabitq-fwht-unsafe-reduction
fixture: n/a
storage_format: n/a
rerank_mode: n/a
surface: shared-quant-kernel
index_surface: n/a
shared_table_surface: n/a
---

# Artifact Manifest

## Artifacts

### `block-count-after.log`

- Command: `make unsafe-block-count PATHS='src/quant/hadamard.rs'`
- Timestamp: 2026-05-20 01:00:36 -07:00
- Exit code: 0
- Key line: `43 src/quant/hadamard.rs`

### `rustfmt-touched-check.log`

- Command: `rustfmt --edition 2021 --check src/quant/hadamard.rs`
- Timestamp: 2026-05-20 01:00:36 -07:00
- Exit code: 0
- Result: touched-file rustfmt check passed. The log contains only existing stable-rustfmt warnings about unstable import options.

### `git-diff-check.log`

- Command: `git diff --check`
- Timestamp: 2026-05-20 01:00:36 -07:00
- Exit code: 0
- Result: whitespace check passed.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Timestamp: 2026-05-20 01:00:40 -07:00
- Exit code: 0
- Result: PG18/bench cargo check passed.
- Residual warnings are existing unused import/export warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

### `cargo-test-hadamard-lib-pg18.log`

- Command: `cargo test hadamard --lib --no-default-features --features pg18`
- Timestamp: 2026-05-20 01:00:45 -07:00
- Exit code: 127
- Result: lib test compiled, then the local test binary failed before running the filtered tests with `undefined symbol: CacheRegisterRelcacheCallback`.
