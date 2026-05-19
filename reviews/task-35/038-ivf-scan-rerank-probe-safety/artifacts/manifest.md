# Task 35 Packet 038 Artifact Manifest

- Head SHA: `b2b6b99e60ea44c0f6147dcd11744f71c8c3acad`
- Task bucket: `reviews/task-35/038-ivf-scan-rerank-probe-safety`
- Timestamp: `2026-05-19T06:35:01Z`
- Lane: Task 35 unsafe-comment burndown
- Fixture / storage format / rerank mode: not applicable; static safety
  documentation and baseline accounting only
- Table surface: not applicable

## Artifacts

### `unsafe-baseline-report-before.log`

- Command: `scripts/report_unsafe_comment_baseline.sh`
- Key result: before update, the global baseline still contained 2789 entries
  and `src/am/ec_ivf/scan.rs` contained 69 entries.

### `ivf-scan-baseline-before.log`

- Command:
  `grep -n '^src/am/ec_ivf/scan.rs:' scripts/unsafe_comment_baseline.txt`
- Key result: captured the pre-slice `src/am/ec_ivf/scan.rs` unsafe-comment
  baseline entries used to choose the rerank/probe cluster.

### `unsafe-audit-before-baseline-update.log`

- Command: `scripts/audit_unsafe_comments.sh`
- Key result: captured the expected pre-baseline-update audit output after the
  source comments changed.

### `ivf-scan-diff-before-baseline.patch`

- Command: `git diff -- src/am/ec_ivf/scan.rs`
- Key result: captured the source-only comment changes before regenerating the
  unsafe-comment baseline.

### `unsafe-baseline-update.log`

- Command: `scripts/update_unsafe_comment_baseline.sh`
- Key result: regenerated the unsafe-comment baseline after the source comment
  updates.

### `cargo-fmt.log`

- Command: `cargo fmt --all`
- Key result: formatting completed. Unrelated formatter churn in
  `hardening/careful/src/lib.rs` and `src/quant/simd.rs` was restored before
  commit.

### `unsafe-baseline-update-after-fmt.log`

- Command: `scripts/update_unsafe_comment_baseline.sh`
- Key result: regenerated the unsafe-comment baseline again after formatting.

### `unsafe-audit-after.log`

- Command: `scripts/audit_unsafe_comments.sh`
- Key result: empty output; no unsafe-comment audit drift remained.

### `unsafe-baseline-report-after.log`

- Command: `scripts/report_unsafe_comment_baseline.sh`
- Key result: after update, the global baseline contained 2763 entries and
  `src/am/ec_ivf/scan.rs` contained 43 entries.

### `ivf-scan-baseline-after.log`

- Command:
  `grep -n '^src/am/ec_ivf/scan.rs:' scripts/unsafe_comment_baseline.txt`
- Key result: captured the remaining 43 `src/am/ec_ivf/scan.rs` baseline
  entries after this slice.

### `unsafe-baseline-after-count.log`

- Command:
  `grep -c '^' scripts/unsafe_comment_baseline.txt` and
  `grep -c '^src/am/ec_ivf/scan.rs:' scripts/unsafe_comment_baseline.txt`
- Key result: global count `2763`; IVF scan count `43`.

### `git-diff-check.log`

- Command: `git diff --exit-code -- scripts/unsafe_comment_baseline.txt`
- Key result: empty output; regenerated baseline was clean.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Key result: completed successfully with the known unrelated warnings for
  `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs` and unused
  SPIRE re-exports in `src/am/mod.rs`.

### `final-diff.patch`

- Command:
  `git diff HEAD -- src/am/ec_ivf/scan.rs scripts/unsafe_comment_baseline.txt`
- Key result: final source and baseline diff for the packet.
