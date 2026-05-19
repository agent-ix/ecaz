# Task 35 Packet 095 Artifact Manifest

- Head SHA: `85ed2b9563c7cb553544fc902692847540012b54`
- Task bucket: `reviews/task-35/`
- Packet path: `reviews/task-35/095-hnsw-parallel-dsm-insert-search-safety/`
- Timestamp: `2026-05-19T11:27:16Z`
- Lane / fixture / storage format / rerank mode: unsafe-comment audit only; not applicable
- One-index-per-table vs shared-table surface: not applicable; no database benchmark or SQL fixture was run

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `bash scripts/unsafe_baseline_report.sh`
- Purpose: capture the pre-slice unsafe baseline summary.
- Key result: global baseline had `1214` entries across `42` files.

### `hnsw-build-parallel-baseline-before.log`

- Command: `rg '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt; printf 'entries: '; rg -c '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt`
- Purpose: capture the pre-slice file-local baseline entries.
- Key result: `entries: 184`.

### `unsafe-audit-before.log`

- Command: `bash scripts/check_unsafe_comments.sh`
- Purpose: verify the starting baseline was internally consistent.
- Key result: command exited `0`.

### `unsafe-baseline-update.log`

- Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
- Purpose: first baseline update after documenting the DSM insert/search path.
- Key result: `wrote scripts/unsafe_comment_baseline.txt with 1172 entries`.

### `hnsw-build-parallel-baseline-after-update.log`

- Command: `rg '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt; printf 'entries: '; rg -c '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt`
- Purpose: confirm the target file-local baseline after the first update.
- Key result: `entries: 142`.

### `diff-before-format.patch`

- Command: `git -c color.ui=false diff -- src/am/ec_hnsw/build_parallel.rs scripts/unsafe_comment_baseline.txt`
- Purpose: preserve the pre-format code and baseline diff.

### `cargo-fmt.log`

- Command: `cargo fmt --all`
- Purpose: format the initial slice.
- Key result: command exited `0`; rustfmt emitted the repo's existing stable-toolchain warnings for unstable import options.

### `unsafe-baseline-update-after-fmt.log`

- Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
- Purpose: refresh the baseline after formatting.
- Key result: `wrote scripts/unsafe_comment_baseline.txt with 1172 entries`.

### `cargo-fmt-2.log`

- Command: `cargo fmt --all`
- Purpose: format the small cleanup that moved comments out of macro/arithmetic expressions.
- Key result: command exited `0`; rustfmt emitted the repo's existing stable-toolchain warnings for unstable import options.

### `unsafe-baseline-update-after-cleanup.log`

- Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
- Purpose: refresh the baseline after replacing expression-inline comments with local variables.
- Key result: `wrote scripts/unsafe_comment_baseline.txt with 1171 entries`.

### `unsafe-audit-after.log`

- Command: `bash scripts/check_unsafe_comments.sh`
- Purpose: verify all remaining unsafe sites match the updated baseline.
- Key result: command exited `0`.

### `unsafe-baseline-report-after.log`

- Command: `bash scripts/unsafe_baseline_report.sh`
- Purpose: capture the post-slice unsafe baseline summary.
- Key result: global baseline has `1171` entries across `42` files.

### `hnsw-build-parallel-baseline-after.log`

- Command: `rg '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt; printf 'entries: '; rg -c '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt`
- Purpose: capture the final file-local baseline count.
- Key result: `entries: 141`.

### `git-diff-check.log`

- Command: `git diff --check`
- Purpose: verify the working diff had no whitespace errors before commit.
- Key result: command exited `0`.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: compile-check the touched Rust code under the primary PG18 feature surface.
- Key result: command exited `0`; existing unrelated warnings remain for `EC_PARALLEL_WORKER_SLOT_CLAIMED` and SPIRE re-exports.

### `final-diff.patch`

- Command: `git -c color.ui=false diff -- src/am/ec_hnsw/build_parallel.rs scripts/unsafe_comment_baseline.txt`
- Purpose: preserve the final code and baseline diff reviewed in this packet.
