# Review Request: HNSW Frontier Refill Safety

Head: `5aacdb2b46f344187f0bf140fe12d979b632c8b9`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/017-hnsw-frontier-refill-safety/request.md`
- `reviews/task-35/017-hnsw-frontier-refill-safety/artifacts/*`

What changed:
- Documented synchronous callback access in test/pg_test bootstrap trace and
  discovered-candidate seeding.
- Documented grouped frontier-head exact refinement, including candidate graph
  element loading and exact/grouped scoring dispatch.
- Documented layer-0 refill and visible-seed top-up helpers, covering live
  relation, scan graph storage, scan opaque callback state, and callback TID set
  mutation.
- Documented the refill-after-consume boundary and the public
  `consume_and_refill_bootstrap_frontier` test helper.

Baseline result:
- Start: 3,131 entries across 102 files.
- End: 3,115 entries across 102 files.
- Net reduction: 16 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 60 entries to 44 entries.

Review focus:
- Confirm the callback safety comments accurately describe synchronous closure
  execution and live `opaque_ptr` use.
- Confirm frontier-head refinement comments cover the candidate origin and exact
  scoring state without overstating graph storage guarantees.
- Confirm the test/pg_test refill helpers remain behaviorally unchanged.

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
- Runtime tests skipped under the Task 35 policy. This is a doc-only slice over
  production frontier refinement and test/pg_test refill helpers; validation
  used the unsafe audit, formatting, diff check, and PG18 cargo check.
