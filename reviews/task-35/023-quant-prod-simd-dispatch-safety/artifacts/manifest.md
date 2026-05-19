# Artifact Manifest

Head SHA: `72ebc9ca7acb544a11d6b8a90c9046834ece9034`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/023-quant-prod-simd-dispatch-safety`

Timestamp: `2026-05-19T05:21:28Z`

Surface:
- Quant product-quantizer SIMD dispatch and test-only decode harness.

Artifacts:
- `unsafe-baseline-report-before.log`
  - command: `make unsafe-baseline-report`
  - result: 2,989 entries across 100 files.
- `prod-baseline-before.log`
  - command: `grep -c '^src/quant/prod.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 12.
- `quant-baseline-before.log`
  - command: `grep -c '^src/quant/' scripts/unsafe_comment_baseline.txt`
  - result: 12.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: passes after source comments, before baseline refresh.
- `prod-diff-before-baseline.patch`
  - command: `git diff -- src/quant/prod.rs`
  - result: source-only `prod.rs` SAFETY comment diff before baseline refresh.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - result: writes 2,977 entries.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - result: passes.
- `unsafe-audit-after-fmt-before-restore.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: captured during fmt/restoration workflow.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: passes.
- `unsafe-baseline-report-after.log`
  - command: `make unsafe-baseline-report`
  - result: 2,977 entries across 99 files.
- `prod-baseline-after.log`
  - command: `grep -c '^src/quant/prod.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 0.
- `quant-baseline-after.log`
  - command: `grep -c '^src/quant/' scripts/unsafe_comment_baseline.txt`
  - result: 0.
- `unsafe-baseline-after-count.log`
  - command: `grep -c '^' scripts/unsafe_comment_baseline.txt`
  - result: 2,977.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passes with existing unused-import warnings.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passes.
- `final-diff.patch`
  - command: `git diff -- src/quant/prod.rs scripts/unsafe_comment_baseline.txt`
  - result: final source and baseline diff before commit.

Notes:
- This packet does not use a lane / fixture / storage format / rerank mode.
- This packet does not use isolated one-index-per-table or shared-table
  benchmark surfaces.
