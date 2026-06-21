# Task 118 HNSW Candidate Pool Diagnostic Fix Artifacts

- Head SHA: `59858d090`
- Task bucket: `reviews/task-118/`
- Packet path: `reviews/task-118/005-hnsw-candidate-pool-diagnostic-fix/`
- Timestamp: 2026-06-21 America/Los_Angeles
- Surface: PG18 local scratch database `tqvector_bench`, existing 10k TurboQuant HNSW source-build table from the in-progress Task 118 suite.
- Isolation: one index per table.

## Artifacts

- `cargo-test-ecaz-cli-hnsw-diagnostic-candidate-pool.log`
  - Command: `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Result: 21 passed, 0 failed.

- `cargo-pgrx-install-pg18-pgtest-candidate-pool.log`
  - Command: `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
  - Result: installed `ecaz` with pg_test diagnostics into the local PG18 pgrx install.

- `frontier-10k-hnsw-turboquant-candidate-pool-smoke5.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench bench hnsw-frontier --prefix task118_real10k_hnsw_turboquant --m 16 --sweep "40,64,100,128,160,200" --queries-limit 5 --log-output reviews/task-118/005-hnsw-candidate-pool-diagnostic-fix/artifacts/frontier-10k-hnsw-turboquant-candidate-pool-smoke5.log --jsonl-output reviews/task-118/005-hnsw-candidate-pool-diagnostic-fix/artifacts/frontier-10k-hnsw-turboquant-candidate-pool-smoke5.jsonl`
  - Key result lines:
    - `ef_search=40`: truth@10 in frontier `0.6200`, frontier `40.0`, exact rerank `40.0`, dropped before exact `0.0`.
    - `ef_search=128`: truth@10 in frontier `1.0000`, frontier `128.0`, exact rerank `128.0`, dropped before exact `0.0`.
    - `ef_search=200`: truth@10 in frontier `1.0000`, truth@100 in frontier `0.9320`, frontier `200.0`, exact rerank `200.0`, dropped before exact `0.0`.

- `frontier-10k-hnsw-turboquant-candidate-pool-smoke5.jsonl`
  - Per-query rows for the smoke run above.
  - Confirms `pre_final_frontier_size`, `frontier_row_indices`, exact score/rank arrays, and containment counters are now derived from the emitted graph candidate pool before caller-side top-k truncation.

## Notes

This is not the final Task 118 attribution matrix. It validates the diagnostic correction that the final matrix depends on.
