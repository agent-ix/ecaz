# Review Request: HNSW Graph Tuple Loader Safety

Head: `5b84fd25e88ff375d5eb015ebeca99d16f6f97c5`

Scope:
- `src/am/ec_hnsw/graph.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/020-hnsw-graph-tuple-loader-safety/request.md`
- `reviews/task-35/020-hnsw-graph-tuple-loader-safety/artifacts/*`

What changed:
- Documented storage descriptor reloption lookup against live index metadata.
- Documented scalar, TurboQuant hot/cold, PqFastScan grouped hot, rerank, and
  grouped codebook tuple loader boundaries.
- Documented tuple-ref callback helpers for relation-backed page reads and the
  PG18 pinned-buffer graph tuple path.

Baseline result:
- Start: 3,071 entries across 101 files.
- End: 3,050 entries across 101 files.
- Net reduction: 21 baseline entries.
- `src/am/ec_hnsw/graph.rs` start/end: 56 entries to 35 entries.

Review focus:
- Confirm the comments distinguish live relation/page tuple invariants from
  storage-format validation handled elsewhere.
- Confirm cold rerank payload comments correctly tie `reranktid` to the decoded
  hot tuple rather than assuming a broader relation invariant.
- Confirm PG18 pinned-buffer comments accurately describe caller-owned pin/lock
  lifetime.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before-baseline-update.log`
  - result: expected failure from line drift before updating the baseline.
- `git diff -- src/am/ec_hnsw/graph.rs`
  - artifact: `artifacts/graph-rs-diff-before-baseline.patch`
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
- Runtime tests skipped under the Task 35 policy. This is a documentation-only
  graph tuple-loader slice; validation used the unsafe audit, formatting, diff
  check, and PG18 cargo check.
