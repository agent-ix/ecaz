---
task: 50
packet: reviews/task-50/017-unsafe-facade-contract-followup
head_sha: 3051e991f89413d7c25c8a1b612c0d69b6b5cc2b
code_commits:
  - 5878b6e3 Restore unsafe facade contracts
  - 3051e991 Clean HNSW facade helper lifetimes
generated_at: 2026-05-20T00:55:59-07:00
lane: unsafe-facade-review-followup
fixture: n/a
storage_format: n/a
rerank_mode: n/a
surface: code-review
index_surface: n/a
shared_table_surface: n/a
---

# Artifact Manifest

This packet responds to reviewer feedback on the Task 50 unsafe facade packets:

- `reviews/task-50/012-hnsw-scan-debug-facade/feedback/2026-05-20-01-reviewer.md`
- `reviews/task-50/013-ivf-scan-storage-debug-facade/feedback/2026-05-20-01-reviewer.md`
- `reviews/task-50/014-spire-snapshot-live-relation-facade/feedback/2026-05-20-01-reviewer.md`
- `reviews/task-50/015-hnsw-scan-opaque-accessors/feedback/2026-05-20-01-reviewer.md`

## Artifacts

### `block-count-after.log`

- Command: `make unsafe-block-count PATHS='src/am/ec_hnsw/scan.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_ivf/scan.rs src/am/ec_spire/coordinator/snapshots.rs'`
- Timestamp: 2026-05-20 00:53:39 -07:00
- Exit code: 0
- Key lines:
  - `158 src/am/ec_hnsw/scan.rs`
  - `135 src/am/ec_hnsw/scan_debug.rs`
  - `69 src/am/ec_ivf/scan.rs`
  - `42 src/am/ec_spire/coordinator/snapshots.rs`

### `rustfmt-touched-check.log`

- Command: `rustfmt --edition 2021 --check src/am/ec_hnsw/scan.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_ivf/scan.rs src/am/ec_spire/coordinator/snapshots.rs`
- Timestamp: 2026-05-20 00:53:43 -07:00
- Exit code: 0
- Result: touched-file rustfmt check passed. The log contains only existing stable-rustfmt warnings about unstable import options.

### `git-diff-check.log`

- Command: `git diff --check`
- Timestamp: 2026-05-20 00:53:45 -07:00
- Exit code: 0
- Result: whitespace check passed.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Timestamp: 2026-05-20 00:53:47 -07:00
- Exit code: 0
- Result: PG18/bench cargo check passed.
- Residual warnings are existing unused import/export warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

### `cargo-clippy-pg18.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Timestamp: 2026-05-20 00:54:03 -07:00
- Exit code: 101
- Result: clippy still fails on the existing repo-wide backlog.
- Follow-up audit: searching the artifact for the touched facade files and helper names returned no matches after commit `3051e991`, so the final clippy failure is not from this follow-up's touched files.
