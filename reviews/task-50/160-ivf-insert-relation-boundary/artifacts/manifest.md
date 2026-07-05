# Task 50 Packet 160 Artifact Manifest

- head SHA: `40aa6ce22afef20521376015fc468bea24a71c8d`
- task bucket: `reviews/task-50/160-ivf-insert-relation-boundary`
- timestamp: `2026-05-20T21:34:26-07:00`
- lane: IVF unsafe-boundary burndown / soundness-audit follow-up
- fixture: Rust compile/static validation
- storage format: IVF insert path, including PQFastScan reencode path
- rerank mode: n/a
- isolated one-index-per-table or shared-table surface: n/a

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/160-ivf-insert-relation-boundary/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile ...`; existing warning remains in `src/am/mod.rs` for unused DML frontdoor imports.
- `git-diff-check.log`
  - command: `script -q -c "git diff --check" reviews/task-50/160-ivf-insert-relation-boundary/artifacts/git-diff-check.log`
  - result: passed with no output.
- `unsafe-block-count.log`
  - command: `script -q -c "make unsafe-block-count" reviews/task-50/160-ivf-insert-relation-boundary/artifacts/unsafe-block-count.log`
  - result: passed.
  - key lines: `4 src/am/ec_ivf/insert.rs`; prior packet 159 had `12 src/am/ec_ivf/insert.rs`.
