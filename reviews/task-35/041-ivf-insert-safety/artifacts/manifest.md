# Task 35 Packet 041 Artifact Manifest

- Head SHA: `c3af01e076cbb303d22bf274d06b032d51f95de3`
- Task bucket: `reviews/task-35/041-ivf-insert-safety`
- Timestamp: `2026-05-19T06:51:30Z`
- Lane: Task 35 unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable; static safety
  documentation and baseline accounting only
- Table surface: not applicable

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `bash scripts/unsafe_baseline_report.sh`
- Key result: before update, the global baseline contained 2694 entries and
  `src/am/ec_ivf/insert.rs` contained 21 entries.

### `ivf-insert-baseline-before.log`

- Command:
  `grep -n '^src/am/ec_ivf/insert.rs:' scripts/unsafe_comment_baseline.txt`
- Key result: captured the 21 remaining `src/am/ec_ivf/insert.rs` baseline
  entries.

### `unsafe-audit-before-baseline-update.log`

- Command: `bash scripts/check_unsafe_comments.sh`
- Key result: completed with exit code 0 before baseline regeneration because
  this slice only reduced already-baselined unsafe-comment debt.

### `ivf-insert-diff-before-baseline.patch`

- Command: `git diff -- src/am/ec_ivf/insert.rs`
- Key result: captured the source-only comment changes before regenerating the
  unsafe-comment baseline.

### `unsafe-baseline-update.log`

- Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
- Key result: regenerated `scripts/unsafe_comment_baseline.txt` with 2673
  entries.

### `cargo-fmt.log`

- Command: `cargo fmt --all`
- Key result: formatting completed. Unrelated formatter churn in
  `hardening/careful/src/lib.rs` and `src/quant/simd.rs` was restored before
  commit.

### `unsafe-baseline-update-after-fmt.log`

- Command: `bash scripts/check_unsafe_comments.sh --update-baseline`
- Key result: regenerated the unsafe-comment baseline again after formatting,
  still with 2673 entries.

### `unsafe-audit-after.log`

- Command: `bash scripts/check_unsafe_comments.sh`
- Key result: command exited 0 with no diagnostic output.

### `unsafe-baseline-report-after.log`

- Command: `bash scripts/unsafe_baseline_report.sh`
- Key result: after update, the global baseline contained 2673 entries and
  `src/am/ec_ivf/insert.rs` no longer appeared in the report.

### `ivf-insert-baseline-after.log`

- Command:
  `awk 'BEGIN{n=0} /^src\/am\/ec_ivf\/insert.rs:/{print NR ":" $0; n++} END{print "entries: " n}' scripts/unsafe_comment_baseline.txt`
- Key result: `entries: 0`.

### `unsafe-baseline-after-count.log`

- Command:
  `awk 'BEGIN{insert=0} /^src\/am\/ec_ivf\/insert.rs:/{insert++} {total++} END{print "global: " total; print "src/am/ec_ivf/insert.rs: " insert}' scripts/unsafe_comment_baseline.txt`
- Key result: global count `2673`; IVF insert count `0`.

### `git-diff-check.log`

- Command: `git diff --check`
- Key result: command exited 0 with no diagnostic output.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: completed successfully with the known unrelated warnings for
  `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs` and unused
  SPIRE re-exports in `src/am/mod.rs`.

### `final-diff.patch`

- Command:
  `git diff -- src/am/ec_ivf/insert.rs scripts/unsafe_comment_baseline.txt`
- Key result: final source and baseline diff for the packet.
