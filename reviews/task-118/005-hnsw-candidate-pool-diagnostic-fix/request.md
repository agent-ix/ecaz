# Review Request: Task 118 HNSW Candidate Pool Diagnostic Fix

## Scope

This checkpoint fixes the Task 118 HNSW containment diagnostic before continuing the full attribution matrix.

The prior diagnostic used the leftover visible frontier snapshot for `truth_top10_in_frontier` and related arrays. That could report zero containment even when the graph scan later emitted the exact truth rows before SQL-level top-k truncation. The fix makes the containment rows, exact score arrays, rank arrays, and `pre_final_frontier_size` describe the retained graph candidate pool emitted by the AM before caller-side truncation.

The CLI diagnostic calls are also schema-qualified to `tests.*` with explicit argument casts so pg_test exports are found reliably from `ecaz bench hnsw-frontier` and `ecaz bench hnsw-score-correlation`.

## Code

- Commit under review: `59858d090` (`Fix HNSW containment candidate pool diagnostic`)
- Files touched:
  - `src/am/ec_hnsw/scan_debug.rs`
  - `src/tests/ec_hnsw_recall_helpers.rs`
  - `crates/ecaz-cli/src/commands/bench/hnsw_frontier.rs`
  - `crates/ecaz-cli/src/commands/bench/hnsw_score_correlation.rs`

## Validation

- `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Artifact: `artifacts/cargo-test-ecaz-cli-hnsw-diagnostic-candidate-pool.log`
  - Result: 21 passed, 0 failed.

- PG18 pg_test install:
  - Artifact: `artifacts/cargo-pgrx-install-pg18-pgtest-candidate-pool.log`
  - Result: extension installed successfully into the local PG18 pgrx tree.

- 10k TurboQuant containment smoke, 5 queries across ef sweep:
  - Artifacts:
    - `artifacts/frontier-10k-hnsw-turboquant-candidate-pool-smoke5.log`
    - `artifacts/frontier-10k-hnsw-turboquant-candidate-pool-smoke5.jsonl`
  - Key result: `frontier` now equals the retained candidate pool size (`40`, `64`, `100`, `128`, `160`, `200`) and exact rerank count matches it for this source-build TurboQuant surface.
  - Key result: truth@10 containment rises with ef, reaching `1.0000` at `ef_search=128` and `200`.

## Remaining Task 118 Work

This packet does not close Task 118. The full attribution matrix still needs recall, latency, storage, containment, score correlation, and build-source A/B evidence across 10k/50k/100k for TurboQuant, PqFastScan, and RaBitQ.
