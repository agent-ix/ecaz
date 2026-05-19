# Review Request: Common Parallel Production Safety

Head: `ce380578c4cf470140d51eb7300b7377f8813a62`

Scope:
- `src/am/common/parallel.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/010-common-parallel-production-safety/request.md`
- `reviews/task-35/010-common-parallel-production-safety/artifacts/*`

What changed:
- Documented the production unsafe boundaries in `src/am/common/parallel.rs`
  for AM-private parallel scan descriptor layout, worker slot pointer
  derivation, PostgreSQL parallel scan offsets, pgrx callback guards, and
  shared coordinator/slot reads and writes.
- Consolidated the three raw descriptor-header reads in
  `reset_parallel_scan_layout` into one unsafe block with a single local
  contract.
- Left the remaining `src/am/common/parallel.rs` baseline entries in the test
  harness for a later Task 35 test-only slice.

Baseline result:
- Start: 3,448 entries across 103 files.
- End: 3,404 entries across 103 files.
- Net reduction: 44 baseline entries.
- `src/am/common/parallel.rs` start/end: 119 entries to 75 entries.

Review focus:
- Confirm the shared-memory layout comments state the actual descriptor
  invariants: state header first, coordinator after the MAXALIGN state header,
  worker slots after the recorded coordinator span, and bounded slot indices.
- Confirm the PostgreSQL callback comments distinguish C-boundary guarding from
  the separate raw pointer validity checks.
- Confirm the remaining `src/am/common/parallel.rs` baseline entries are
  intentionally test harness entries, not missed production paths.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before-baseline-update.log`
  - result: expected failure due shifted `src/am/common/parallel.rs` test line
    numbers before the baseline refresh.
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/unsafe-baseline-update.log`
- `make unsafe-baseline-report` after initial baseline update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh` after initial baseline update
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passed with no output.
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh --update-baseline` after formatting
  - artifact: `artifacts/unsafe-baseline-update-after-fmt.log`
- `make unsafe-baseline-report` final
  - artifact: `artifacts/unsafe-baseline-final.log`
- `bash scripts/check_unsafe_comments.sh` final
  - artifact: `artifacts/unsafe-audit-final.log`
  - result: passed with no output.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passed with existing warnings from `src/am/common/parallel.rs` and
    `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests skipped:
- PostgreSQL runtime tests were not run. This packet changes unsafe
  documentation and consolidates a local raw-read block without changing scan
  behavior.
