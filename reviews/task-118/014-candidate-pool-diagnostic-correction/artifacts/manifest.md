# Task 118 Packet 014 Artifact Manifest

- head SHA: `df7ff2a0324929bd385e710ed97807be971773df`
- task bucket: `reviews/task-118/014-candidate-pool-diagnostic-correction`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: HNSW Task 118 candidate-pool diagnostic correction; smoke uses 10k TurboQuant source-build at `ef_search=200`.
- isolated surface: one-index source-build smoke on AMD host; not final Intel benchmark evidence.

## Artifacts

### `cargo-check-pg18-pgtest.log`

- command: `cargo check --features 'pg18 pg_test' --no-default-features`
- result: passed

### `cargo-pgrx-install-pg18-pgtest.log`

- command: `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- result: passed
- purpose: install the corrected pg_test diagnostic implementation into the local PG18 scratch cluster for the smoke run.

### `frontier-smoke-10k-turboquant-ef200.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench bench hnsw-frontier --prefix task118_real10k_hnsw_turboquant --m 16 --sweep 200 --queries-limit 5 --log-output reviews/task-118/014-candidate-pool-diagnostic-correction/artifacts/frontier-smoke-10k-turboquant-ef200.log --jsonl-output reviews/task-118/014-candidate-pool-diagnostic-correction/artifacts/frontier-smoke-10k-turboquant-ef200.jsonl`
- key result: `truth@10 in frontier=1.0000`, `frontier=200.0`, `emitted=200.0`, `exact rerank=200.0`, `dropped before exact=0.0`.

### `frontier-smoke-10k-turboquant-ef200-summary.md`

- source: derived from the smoke JSONL using `jq -r '[.ef_search, .query_index, .pre_final_frontier_size, (.frontier_row_indices|length), .final_emitted_count, (.final_emitted_row_indices|length), .truth_top10_in_frontier] | @tsv'`.
- key result: all five smoke rows have `pre_final_frontier_size=200`, `frontier_row_count=200`, `final_emitted_count=200`, `final_emitted_row_count=200`, and `truth_top10_in_frontier=10`.

## Commit Hygiene

The raw per-query smoke JSONL is intentionally not a committed artifact. It was
used only to derive `frontier-smoke-10k-turboquant-ef200-summary.tsv`.
