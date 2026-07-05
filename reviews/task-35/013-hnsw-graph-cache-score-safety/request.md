# Review Request: HNSW Graph Cache and Grouped Score Safety

Head: `68d4bba75018bad2f1f10f5c5a673cdabe686189`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/013-hnsw-graph-cache-score-safety/request.md`
- `reviews/task-35/013-hnsw-graph-cache-score-safety/artifacts/*`

What changed:
- Documented HNSW grouped traversal and rerank boundaries: cached graph
  element/buffer loaders, exact/approx/binary grouped scoring dispatch,
  grouped heap rerank scoring, grouped candidate context scoring,
  cached graph neighbors and adjacency.
- Migrated `cached_graph_element_from_buffer` and `cached_graph_neighbors`
  cache-hit branches to use the safe `graph_element_cache_mut` /
  `graph_neighbor_cache_mut` helpers introduced in packet 012, so the
  raw cache-pointer deref is no longer repeated at each hit site.
- Replaced two `&*opaque.cached_quantizer` raw derefs with the safe
  `cached_quantizer_ref` helper in `score_grouped_rerank_payload_from_scan_state`
  and `score_grouped_candidate_context_binary`.
- Added `// SAFETY:` contracts at every retained `unsafe { ... }` site in
  the grouped traversal / rerank scoring path.

Baseline result:
- Start: 3,264 entries across 102 files.
- End: 3,212 entries across 102 files.
- Net reduction: 52 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 193 entries to 141 entries.

Review focus:
- Confirm the new `// SAFETY:` contracts accurately describe the live
  lifetimes for the scan opaque, index relation, prepared query, cached
  quantizer, grouped heap rerank state, and cached graph element pointers.
- Confirm the cache-helper migration in
  `cached_graph_element_from_buffer` and `cached_graph_neighbors`
  preserves cache-hit semantics: the `.cloned()` on the cache lookup
  matches the previous `Arc::clone(element)` return path.
- Confirm the `cached_quantizer_ref` substitution preserves the existing
  "scan state is missing cached quantizer" error message.

Validation:
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before-baseline-update.log`
  - result: expected failure from line drift before updating the baseline.
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/unsafe-baseline-update.log`
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh` after baseline update
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passed with no output.
- `make unsafe-baseline-report` after baseline update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passed; pre-existing unused-import warnings in `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.
- `bash scripts/check_unsafe_comments.sh` final
  - artifact: `artifacts/unsafe-audit-final.log`
  - result: passed with no output.
- `make unsafe-baseline-report` final
  - artifact: `artifacts/unsafe-baseline-final.log`

Tests:
- Per CLAUDE.md coder workflow, runtime tests skipped. The slice is
  doc/refactor-only on a path that does not change call semantics:
  cache-hit branches return the same `Arc` they previously returned,
  and `cached_quantizer_ref` preserves the original missing-quantizer
  error path. Verified statically with `cargo check`.
