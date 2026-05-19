# Review Request: HNSW Linear Output Safety

Head: `32d33db43fecb87a12a05cf3a0fa69fc4ddb9dcc`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/018-hnsw-linear-output-safety/request.md`
- `reviews/task-35/018-hnsw-linear-output-safety/artifacts/*`

What changed:
- Documented graph traversal prefetch refresh and graph-phase tuple emission.
- Documented linear fallback result selection through PG18 read streams and the
  non-PG18 direct buffer path.
- Documented linear scan tuple-byte decoding from share-locked buffers.
- Replaced the raw cached-quantizer dereference in scan element scoring with
  `cached_quantizer_ref`, preserving the existing missing-quantizer error.
- Documented TurboQuant prepared-query pointer reads, prepared-query scoring,
  heap TID output, order-by score allocation/write, and order-by clearing.

Baseline result:
- Start: 3,115 entries across 102 files.
- End: 3,096 entries across 102 files.
- Net reduction: 19 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 44 entries to 25 entries.

Review focus:
- Confirm read-stream and buffer-lock comments correctly describe PG18 and
  non-PG18 linear fallback paths.
- Confirm the `cached_quantizer_ref` substitution preserves scan scoring
  behavior and error text.
- Confirm PostgreSQL scan descriptor output comments accurately cover
  `xs_heaptid`, `xs_orderbyvals`, and `xs_orderbynulls` writes.

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
- Runtime tests skipped under the Task 35 policy. This is a doc/refactor slice
  over scan output and linear fallback selection; validation used the unsafe
  audit, formatting, diff check, and PG18 cargo check.
