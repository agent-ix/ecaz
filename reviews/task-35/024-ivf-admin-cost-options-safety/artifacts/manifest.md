# Artifact Manifest

Head SHA: `fdf7a8ee3c8a67aa402aa7468e8d4e45033eb2f7`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/024-ivf-admin-cost-options-safety`

Timestamp: `2026-05-19T05:26:27Z`

Surface:
- IVF admin diagnostics, planner callbacks, and relation options.

Artifacts:
- `unsafe-baseline-report-before.log`
  - command: `make unsafe-baseline-report`
  - result: 2,977 entries across 99 files.
- `ivf-small-baseline-before.log`
  - command: `grep -Ec '^src/am/ec_ivf/(admin|cost|options)\.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 22.
- `admin-baseline-before.log`
  - command: `grep -c '^src/am/ec_ivf/admin.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 10.
- `cost-baseline-before.log`
  - command: `grep -c '^src/am/ec_ivf/cost.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 4.
- `options-baseline-before.log`
  - command: `grep -c '^src/am/ec_ivf/options.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 8.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - result: passes after source comments, before baseline refresh.
- `ivf-small-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_ivf/admin.rs src/am/ec_ivf/cost.rs src/am/ec_ivf/options.rs`
  - result: source-only SAFETY comment diff before baseline refresh.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - result: writes 2,955 entries.
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
  - result: 2,955 entries across 96 files.
- `ivf-small-baseline-after.log`
  - command: `grep -Ec '^src/am/ec_ivf/(admin|cost|options)\.rs:' scripts/unsafe_comment_baseline.txt`
  - result: 0.
- `unsafe-baseline-after-count.log`
  - command: `grep -c '^' scripts/unsafe_comment_baseline.txt`
  - result: 2,955.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passes with existing unused-import warnings.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: passes.
- `final-diff.patch`
  - command: `git diff -- src/am/ec_ivf/admin.rs src/am/ec_ivf/cost.rs src/am/ec_ivf/options.rs scripts/unsafe_comment_baseline.txt`
  - result: final source and baseline diff before commit.

Notes:
- This packet does not use a lane / fixture / storage format / rerank mode.
- This packet does not use isolated one-index-per-table or shared-table
  benchmark surfaces.
