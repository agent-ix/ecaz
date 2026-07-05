# Review Request: HNSW Successor Prefetch Safety

Head: `2757268ed97d6b27ed0623bbe5ccb28a1bfeca11`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/014-hnsw-successor-prefetch-safety/request.md`
- `reviews/task-35/014-hnsw-successor-prefetch-safety/artifacts/*`

What changed:
- Documented PG18 graph prefetch read-stream boundaries in
  `prefetch_graph_buffers`, including stream reset, `read_stream_next_buffer`,
  pinned-buffer guard ownership, and per-buffer block-number data.
- Documented prefetched-buffer graph element loading and fallback graph element
  loading in `cached_graph_element_with_prefetch`.
- Documented successor traversal scoring in
  `cached_scan_successor_candidates_for_layer`, covering cached adjacency,
  per-layer scan configuration reads, grouped/exact candidate scoring, binary
  prefilter score-cache/timing stats, and budgeted grouped scoring.
- Replaced the raw cached-quantizer dereference in the binary successor path
  with `cached_quantizer_ref`, preserving the existing missing-quantizer error.
- Documented the upper-layer greedy-descent successor callback boundary.

Baseline result:
- Start: 3,212 entries across 102 files.
- End: 3,180 entries across 102 files.
- Net reduction: 32 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 141 entries to 109 entries.

Review focus:
- Confirm the PG18 read-stream comments accurately describe stream ownership,
  pinned-buffer ownership, and per-buffer data lifetime.
- Confirm the successor traversal comments accurately describe relation/opaque
  lifetime and candidate ownership through exact, grouped, and binary paths.
- Confirm the `cached_quantizer_ref` substitution preserves the prior runtime
  behavior and error message.

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
  - artifacts: `artifacts/unsafe-baseline-update.log`,
    `artifacts/unsafe-baseline-update-after-fmt.log`
- `bash scripts/check_unsafe_comments.sh` after baseline update
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passed with no output.
- `make unsafe-baseline-report` after baseline update
  - artifacts: `artifacts/unsafe-baseline-after.log`,
    `artifacts/unsafe-baseline-final.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests:
- Runtime tests skipped under the Task 35 policy. This is a doc/refactor-only
  slice; validation used the unsafe audit, formatting, diff check, and PG18
  cargo check.
