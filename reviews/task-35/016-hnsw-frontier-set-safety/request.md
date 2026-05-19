# Review Request: HNSW Frontier Set Safety

Head: `f1df1202e691f1d2eb3fa5d213b3dd1eebbd843c`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/016-hnsw-frontier-set-safety/request.md`
- `reviews/task-35/016-hnsw-frontier-set-safety/artifacts/*`

What changed:
- Documented visible frontier and bootstrap expansion pointer ownership.
- Added scan-owned TID set helpers for reset, insert, and contains operations,
  centralizing the raw `HashSet` pointer invariants for visited, expanded, and
  emitted element sets.
- Reused `drop_boxed_scan_ptr` for visited, expanded, and emitted set teardown
  instead of open-coded `Box::from_raw` frees.
- Documented scan entry candidate initialization, live-entry fallback lookup,
  upper-layer seed traversal, layer-0 successor expansion, and callback access
  from frontier seeding closures.

Baseline result:
- Start: 3,154 entries across 102 files.
- End: 3,131 entries across 102 files.
- Net reduction: 23 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 83 entries to 60 entries.

Review focus:
- Confirm the new TID-set helpers correctly preserve the previous lazy
  allocation, clear, insert, contains, and teardown behavior.
- Confirm the helper-level safety contract is sufficient for visited,
  expanded-source, and emitted-result set slots.
- Confirm the frontier/bootstrap callback comments accurately describe why the
  raw `opaque_ptr` callbacks do not outlive the synchronous frontier mutation.
- Confirm the entry-candidate fallback path comments accurately describe the
  live index relation and scan graph storage invariants.

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
- Runtime tests skipped under the Task 35 policy. This is a narrow
  doc/refactor slice over scan-owned pointer helpers; validation used the unsafe
  audit, formatting, diff check, and PG18 cargo check.
