# Review Request: Common Parallel Test Harness Safety

Head: `8e5f7eba878ca152069d722ad94fb8a2b7602e0a`

Scope:
- `src/am/common/parallel.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/011-common-parallel-test-harness-safety/request.md`
- `reviews/task-35/011-common-parallel-test-harness-safety/artifacts/*`

What changed:
- Reworked the `src/am/common/parallel.rs` test module so repeated unsafe
  setup and assertion call sites go through small test helpers with local
  `// SAFETY:` contracts.
- Converted `test_parallel_scan_desc`, `test_parallel_scan_target`,
  `test_parallel_scan_desc_and_target`, and `SharedParallelScanState::attachment`
  into safe test helpers that contain the raw pointer contracts internally.
- Added wrappers for common test operations: initialize, attach, claim, publish,
  release, read snapshot, reset, worker-slot lookup, coordinator claim count,
  and staged rescan state.
- Documented the test-only `Send`/`Sync` impls for the scoped-thread fixture.
- Removed all remaining `src/am/common/parallel.rs` baseline entries.

Baseline result:
- Start: 3,404 entries across 103 files.
- End: 3,329 entries across 102 files.
- Net reduction: 75 baseline entries.
- `src/am/common/parallel.rs` start/end: 75 entries to 0 entries.

Review focus:
- Confirm the new helpers preserve the previous test behavior and do not hide
  any meaningful per-call invariant that should stay visible at the assertion
  site.
- Confirm the scoped-thread `Send`/`Sync` comments accurately describe the
  storage lifetime and atomic shared-state access.
- Confirm the stale-epoch and rescan tests still exercise the same state
  transitions after moving the raw calls behind wrappers.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before.log`
  - result: passed with no output.
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
- `cargo test --lib parallel_scan --no-run --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-test-parallel-scan-no-run.log`
  - result: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passed with existing warnings from `src/am/common/parallel.rs` and
    `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Runtime test limitation:
- `cargo test --lib parallel_scan --no-default-features --features pg18,bench`
  compiled the test binary, then failed before running the filtered tests with
  `undefined symbol: LockBuffer`.
  - artifact: `artifacts/cargo-test-parallel-scan.log`
  - This appears to be the local PostgreSQL symbol-link limitation for unit
    test execution, not a compile failure in this slice.
