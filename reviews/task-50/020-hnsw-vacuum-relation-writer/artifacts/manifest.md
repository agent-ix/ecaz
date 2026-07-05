---
task: 50
packet: reviews/task-50/020-hnsw-vacuum-relation-writer
head_sha: aacded93f17e34dea06210bc586933efdca94741
code_commit: aacded93 Reduce HNSW vacuum relation unsafe blocks
generated_at: 2026-05-20T01:19:58-07:00
lane: hnsw-vacuum-unsafe-reduction
fixture: n/a
storage_format: scalar, TurboQuant V3, PqFastScan vacuum paths
rerank_mode: n/a
surface: hnsw-vacuum
index_surface: one-index-per-table
shared_table_surface: no
---

# Artifact Manifest

## Artifacts

### `block-count-after.log`

- Command: `make unsafe-block-count PATHS='src/am/ec_hnsw/vacuum.rs'`
- Timestamp: 2026-05-20 01:19:41 -07:00
- Exit code: 0
- Key line: `68 src/am/ec_hnsw/vacuum.rs`

### `rustfmt-touched-check.log`

- Command: `rustfmt --edition 2021 --check src/am/ec_hnsw/vacuum.rs`
- Timestamp: 2026-05-20 01:19:41 -07:00
- Exit code: 0
- Result: touched-file rustfmt check passed. The log contains only existing stable-rustfmt warnings about unstable import options.

### `git-diff-check.log`

- Command: `git diff --check`
- Timestamp: 2026-05-20 01:19:41 -07:00
- Exit code: 0
- Result: whitespace check passed.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Timestamp: 2026-05-20 01:19:47 -07:00
- Exit code: 0
- Result: PG18/bench cargo check passed.
- Residual warnings are existing unused import/export warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.
