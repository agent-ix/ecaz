---
task: 118
packet: reviews/task-118/012-pre-output-frontier-diagnostic
checkpoint_sha: 6ff2d1d3d8aa04edced517497d940c65ea3d6bca
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Pre-Output Frontier Diagnostic Semantics

## Scope

This checkpoint tightens Task 118 candidate-containment evidence.

The previous pg_test helper captured the pre-output visible HNSW frontier, but
the exported `frontier_*` rows used the final emitted stream when computing
truth containment and ranks. That weakened the Task 118 distinction between
"truth row reached the frontier" and "truth row survived final emission".

Commit `6ff2d1d3d8aa04edced517497d940c65ea3d6bca` changes the diagnostic path so:

- `pre_final_frontier_size` comes from the captured visible frontier;
- `frontier_row_indices`, `frontier_approx_scores`, `frontier_exact_scores`,
  `frontier_approx_ranks`, `frontier_exact_ranks`, and truth-containment
  counters use the pre-output frontier candidates;
- `final_emitted_row_indices` remains a separate final-output field;
- a focused synthetic pg_test covers TurboQuant, PqFastScan, and RaBitQ for
  frontier row sizing, emitted row sizing, finite source-f32 score audit, and
  rerank/drop counter consistency.

## Validation

- `cargo check --features 'pg18 pg_test' --no-default-features`
  - Artifact: `artifacts/cargo-check-pg18-pgtest.log`
  - Result: passed

I did not run `cargo pgrx test` on this AMD host. The earlier Task 118 pgrx
runtime attempts stalled at compile/setup phase here, and the Intel desktop is
currently reserved for SPIRE plus final Task 118 measurement.

## Remaining Task 118 Closeout Work

The final Task 118 decision packet still requires Intel 50k and 100k suite
evidence for recall, latency, storage, frontier containment, rerank counters,
score correlation, and source-build vs compressed-build A/B.
