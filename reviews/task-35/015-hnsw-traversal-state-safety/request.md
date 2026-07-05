# Review Request: HNSW Traversal State Safety

Head: `e88aeffd86c225ba51887cd8df13346bba16e26a`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/015-hnsw-traversal-state-safety/request.md`
- `reviews/task-35/015-hnsw-traversal-state-safety/artifacts/*`

What changed:
- Documented graph traversal cursor boundaries for prefetching into the active
  result state and phase-checked scan opaque access.
- Documented graph-vs-linear dispatch in `produce_next_scan_heap_tid`.
- Documented graph traversal materialization boundaries for frontier refinement,
  grouped window buffering, result-state materialization, candidate graph
  element loading, and grouped comparison scoring.
- Reused `drop_boxed_scan_ptr` for scan-owned candidate frontier, bootstrap
  expansion, and graph prefetch state pointers instead of open-coded
  `Box::from_raw` frees.
- Documented scan-owned bootstrap expansion and graph prefetch state resets, as
  well as PG18 read-stream creation/end boundaries.

Baseline result:
- Start: 3,180 entries across 102 files.
- End: 3,154 entries across 102 files.
- Net reduction: 26 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 109 entries to 83 entries.

Review focus:
- Confirm result-state pointer comments accurately describe cursor ownership and
  active result-state lifetime.
- Confirm grouped windowed traversal comments accurately describe candidate
  ownership from frontier consumption through buffered/materialized output.
- Confirm replacing three open-coded scan-owned `Box::from_raw` frees with
  `drop_boxed_scan_ptr` preserves pointer clearing and ownership behavior.
- Confirm PG18 read-stream comments correctly describe relation lifetime and
  callback state ownership.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before-baseline-update.log`
  - result: expected failure from line drift before updating the baseline.
- `git diff -- src/am/ec_hnsw/scan.rs`
  - artifact: `artifacts/scan-rs-diff-before-baseline.patch`
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/unsafe-baseline-update.log`
- `bash scripts/check_unsafe_comments.sh` after baseline update
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passed with no output.
- `make unsafe-baseline-report` after baseline update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests:
- Runtime tests skipped under the Task 35 policy. This is a doc/refactor-only
  slice with three local helper substitutions for existing scan-owned pointer
  frees; validation used the unsafe audit, formatting, diff check, and PG18
  cargo check.
