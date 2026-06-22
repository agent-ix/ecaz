# Task 118 Packet 022 Artifact Manifest

- head SHA: `182ed060f7ef16a33f380e27d7aba20cd15ef565`
- task bucket: `reviews/task-118/022-diagnostic-candidate-pool-honesty`
- generated: `2026-06-21T19:29:26-07:00`
- lane / fixture / storage format / rerank mode: Task 118 HNSW diagnostic
  code-only correction on AMD host; no Intel benchmark lane was run.
- isolated surface: diagnostic SQL row shape and `ecaz bench hnsw-frontier`
  summary/JSON decoding.

## Artifacts

### `cargo-test-ecaz-cli-hnsw-frontier-summary.log`

- command:
  `cargo test -p ecaz-cli hnsw_frontier::tests::summarize_frontier_rows_computes_recall_and_means`
- key result:
  `test commands::bench::hnsw_frontier::tests::summarize_frontier_rows_computes_recall_and_means ... ok`

### `cargo-check-pg18-pgtest.log`

- command:
  `cargo check --no-default-features --features pg18,pg_test`
- key result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`

