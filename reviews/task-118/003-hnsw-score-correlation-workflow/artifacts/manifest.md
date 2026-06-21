---
task: 118
packet: reviews/task-118/003-hnsw-score-correlation-workflow
head_sha: aba7b40e6b483ca20b9887a7c1bd1527f1f55a10
generated_at: 2026-06-21T08:02:38-07:00
---

# Task 118 HNSW Score Correlation Workflow Artifacts

## Summary

This packet validates the third Task 118 checkpoint:

- `ecaz bench hnsw-score-correlation` batches pg_test HNSW approximate/exact score drift diagnostics over loaded query tables.
- The command writes per-query JSONL with compared row ids, approximate ranks, approximate scores, exact scores, and exact ranks.
- `ecaz bench suite` accepts `kind: "hnsw-score-correlation"` and tracks its `.log` and `.jsonl` outputs.
- `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` now expands 54 steps: 10k / 50k / 100k x TurboQuant / PqFastScan / RaBitQ with load, recall, frontier, score-correlation, latency, and storage.

The suite dry-run is shape validation only. It does not execute the benchmark matrix because this checkout does not contain the staged `data/staged-current/` TSV inputs.

## Artifacts

### `cargo-check-pg18-pgtest.log`

- Head SHA: `aba7b40e6b483ca20b9887a7c1bd1527f1f55a10`
- Command:

```bash
cargo check --no-default-features --features pg18,pg_test > reviews/task-118/003-hnsw-score-correlation-workflow/artifacts/cargo-check-pg18-pgtest.log 2>&1
```

- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 13.47s`

### `cargo-test-ecaz-cli-hnsw-score-correlation.log`

- Head SHA: `aba7b40e6b483ca20b9887a7c1bd1527f1f55a10`
- Command:

```bash
cargo test -p ecaz-cli hnsw_score_correlation -- --nocapture > reviews/task-118/003-hnsw-score-correlation-workflow/artifacts/cargo-test-ecaz-cli-hnsw-score-correlation.log 2>&1
```

- Key result: `2 passed; 0 failed; 0 ignored; 0 measured; 411 filtered out`

### `suite-dry-run.log`

- Head SHA: `aba7b40e6b483ca20b9887a7c1bd1527f1f55a10`
- Lane: local dry-run expansion
- Fixture: `data/staged-current/ec_real_{10k,50k,100k}_{corpus,queries,manifest}.tsv/json`
- Storage formats: `turboquant`, `pq_fastscan`, `rabitq`
- Rerank mode: default HNSW exact/source rerank behavior
- Isolated surfaces: one table/index prefix per scale and storage format
- Command:

```bash
cargo run -p ecaz-cli -- bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --dry-run --manifest-output reviews/task-118/003-hnsw-score-correlation-workflow/artifacts/suite-dry-run-manifest.json > reviews/task-118/003-hnsw-score-correlation-workflow/artifacts/suite-dry-run.log 2>&1
```

- Key result: 54 selected suite steps expanded.
- Key score-correlation result: nine `bench hnsw-score-correlation` steps expanded, one for each required scale and storage format.

### `suite-dry-run-manifest.json`

- Head SHA: `aba7b40e6b483ca20b9887a7c1bd1527f1f55a10`
- Command: produced by the `suite-dry-run.log` command above.
- Key result: 54 selected suite records with expected packet-local artifacts for load, recall, frontier, score-correlation, latency, and storage outputs.
