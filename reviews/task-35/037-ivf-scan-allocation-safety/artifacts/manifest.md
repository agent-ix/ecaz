# Artifact Manifest

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/037-ivf-scan-allocation-safety`
Head SHA: `ada94f379ca9a40da2f91d72b9387292b151851f`
Timestamp: `2026-05-19T06:27:24Z`
Lane: unsafe-comment burndown
Fixture/storage/rerank mode: not applicable
Surface isolation: not applicable; source-only unsafe documentation slice

## Artifacts

- `unsafe-baseline-report-before.log`
  - Command: `make unsafe-baseline-report`
  - Key result: baseline started at `2810` entries; `src/am/ec_ivf/scan.rs`
    had `90` entries.
- `ivf-scan-baseline-before.log`
  - Command: `grep -c '^src/am/ec_ivf/scan.rs:' scripts/unsafe_comment_baseline.txt`
  - Key result: `90`.
- `unsafe-audit-before-baseline-update.log`
  - Command: `bash scripts/check_unsafe_comments.sh`
  - Key result: expected stale-baseline state before updating the baseline.
- `ivf-scan-diff-before-baseline.patch`
  - Command: `git diff -- src/am/ec_ivf/scan.rs`
  - Key result: documents the source comments before regenerating the baseline.
- `unsafe-baseline-update.log`
  - Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - Key result: wrote `scripts/unsafe_comment_baseline.txt` with `2789`
    entries.
- `cargo-fmt.log`
  - Command: `cargo fmt --all`
  - Key result: formatting completed; known unrelated format churn was restored
    before final validation.
- `unsafe-baseline-update-after-fmt.log`
  - Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - Key result: final post-format baseline refresh completed.
- `unsafe-audit-after.log`
  - Command: `bash scripts/check_unsafe_comments.sh`
  - Key result: passed; log is empty.
- `unsafe-baseline-report-after.log`
  - Command: `make unsafe-baseline-report`
  - Key result: `2789` entries; `src/am/ec_ivf/scan.rs` has `69` entries.
- `ivf-scan-baseline-after.log`
  - Command: `grep -c '^src/am/ec_ivf/scan.rs:' scripts/unsafe_comment_baseline.txt`
  - Key result: `69`.
- `unsafe-baseline-after-count.log`
  - Command: `grep -c '^' scripts/unsafe_comment_baseline.txt`
  - Key result: `2789`.
- `git-diff-check.log`
  - Command: `git diff --check`
  - Key result: passed; log is empty.
- `cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Key result: passed with existing unrelated warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `final-diff.patch`
  - Command: `git diff -- src/am/ec_ivf/scan.rs scripts/unsafe_comment_baseline.txt`
  - Key result: final source and baseline diff for the code commit.
