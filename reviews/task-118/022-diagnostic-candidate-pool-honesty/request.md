---
task: 118
packet: reviews/task-118/022-diagnostic-candidate-pool-honesty
checkpoint_sha: 182ed060f7ef16a33f380e27d7aba20cd15ef565
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Diagnostic Candidate-Pool Honesty

## Scope

This checkpoint addresses the reviewer feedback in packet 005/021 that the
Task 118 HNSW diagnostic was presenting `frontier` and `final emitted` as
independent evidence even though the current pg_test path observes the same
AM-emitted stream for both.

Changes:

- `debug_gettuple_frontier_containment_report` now explicitly records
  `frontier_equals_final_emitted = true`.
- `candidates_dropped_before_exact_rerank` is now derived from the observable
  candidate-pool size minus exact rerank calls, not from visited nodes.
- `ecaz bench hnsw-frontier` labels the summary as emitted candidate-pool
  containment and exposes `pool == emitted`.
- The pg_test tuple shape and focused assertions were updated so future
  artifacts cannot silently imply a frontier/output separation.

## Validation

- `artifacts/cargo-test-ecaz-cli-hnsw-frontier-summary.log`
  - `cargo test -p ecaz-cli hnsw_frontier::tests::summarize_frontier_rows_computes_recall_and_means`
  - result: passed.
- `artifacts/cargo-check-pg18-pgtest.log`
  - `cargo check --no-default-features --features pg18,pg_test`
  - result: passed.

## Remaining Task 118 Closeout Work

This is an AMD-host code correction only. It does not run or replace the
required Intel production-path 10k/50k/100k recall + latency + storage suite
evidence. The final Task 118 attribution packet should treat existing
diagnostic containment as emitted candidate-pool containment unless a future
instrumentation change captures a genuinely broader pre-rerank frontier.

