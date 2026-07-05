# Review Request: HNSW Scan Tail Safety

Head: `bc51924f88dcce1a005f648acdceb8e204e7585e`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/019-hnsw-scan-tail-safety/request.md`
- `reviews/task-35/019-hnsw-scan-tail-safety/artifacts/*`

What changed:
- Documented scan-owned raw query value slice construction.
- Documented test-only parallel scan descriptor storage setup, AM-private target
  initialization, attachment reads, coordinator pointer reads, and worker-slot
  snapshot reads.
- Documented test-only cache reset assertions for graph element, graph neighbor,
  and score caches.
- Documented the Miri helper that scores through a raw scan opaque pointer.

Baseline result:
- Start: 3,096 entries across 102 files.
- End: 3,071 entries across 101 files.
- Net reduction: 25 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 25 entries to 0 entries.

Review focus:
- Confirm query value slice comments describe the `store_scan_query` allocation
  and dimension invariant.
- Confirm parallel scan test comments accurately describe the aligned test
  storage and AM-private offset setup for PG17/PG18.
- Confirm cache reset assertions and Miri raw-pointer comments remain test-only
  evidence and do not imply broader production invariants.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` after comments, before baseline update
  - artifact: `artifacts/unsafe-audit-after-comments-before-baseline-update.log`
  - result: passed with no output; `scan.rs` was already fully commented before
    the baseline rewrite.
- `git diff -- src/am/ec_hnsw/scan.rs`
  - artifact: `artifacts/scan-rs-diff-before-baseline.patch`
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/unsafe-baseline-update.log`
- `bash scripts/check_unsafe_comments.sh` final
  - artifact: `artifacts/unsafe-audit-final.log`
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
- Runtime tests skipped under the Task 35 policy. This is a documentation-only
  tail cleanup, and several touched sites are test-only. Validation used the
  unsafe audit, formatting, diff check, and PG18 cargo check.
