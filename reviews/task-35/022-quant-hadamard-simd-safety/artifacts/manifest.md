# Artifact Manifest

Head SHA: `ce898c89904c2eedbf088f34e49692b3f7920067`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/022-quant-hadamard-simd-safety`

Timestamp: `2026-05-19T05:16:44Z`

Surface:
- Quant / RABITQ-adjacent Hadamard SIMD FWHT.

Artifacts:
- `unsafe-baseline-report-before.log`
  - command: `make unsafe-baseline-report`
  - result: 3,050 entries across 101 files.
- `hadamard-baseline-before.log`
  - command: `grep -c '^src/quant/hadamard.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 62.
- `ivf-page-baseline-before.log`
  - command: `grep -c '^src/am/ec_ivf/page.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 133.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: fails only on pre-existing `src/am/ec_ivf/page.rs` line drift after
    Hadamard comments were added.
- `hadamard-diff-before-baseline.patch`
  - command: `git diff -- src/quant/hadamard.rs`
  - result: source-only Hadamard SAFETY comment diff before baseline refresh.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - result: writes 2,989 entries.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - result: passes.
- `unsafe-audit-after-fmt-before-refresh.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: captured during fmt/restoration workflow.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: passes.
- `unsafe-baseline-report-after.log`
  - command: `make unsafe-baseline-report`
  - result: 2,989 entries across 100 files.
- `hadamard-baseline-after.log`
  - command: `grep -c '^src/quant/hadamard.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 0.
- `ivf-page-baseline-after.log`
  - command: `grep -c '^src/am/ec_ivf/page.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 134.
- `unsafe-baseline-after-count.log`
  - command: `grep -c '^' scripts/unsafe_comment_baseline.txt`
  - result: 2,989.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passes with existing unused-import warnings.
- `cargo-test-fwht-pg18-bench.log`
  - command: `cargo test --no-default-features --features pg18,bench fwht`
  - result: compiles, then exits before running tests with unresolved
    PostgreSQL symbol `BufferBlocks`.
- `cargo-test-fwht-no-features.log`
  - command: `cargo test --no-default-features fwht`
  - result: blocked by `pgrx-pg-sys` requiring a PostgreSQL feature.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passes.
- `final-diff.patch`
  - command: `git diff -- src/quant/hadamard.rs scripts/unsafe_comment_baseline.txt`
  - result: final source and baseline diff before commit.

Notes:
- This packet does not use a lane / fixture / storage format / rerank mode.
- This packet does not use isolated one-index-per-table or shared-table
  benchmark surfaces.
