---
task: 118
packet: reviews/task-118/002-hnsw-frontier-suite-workflow
head_sha: c143616ed2bc9ac7cffd7361018754247bff6095
generated_at: 2026-06-21T07:50:02-07:00
---

# Task 118 HNSW Frontier Suite Workflow Artifacts

## Summary

This packet validates the second Task 118 checkpoint:

- `ecaz bench hnsw-frontier` exports batched HNSW candidate-frontier containment summaries and JSONL rows.
- `ecaz bench suite` accepts `kind: "hnsw-frontier"` steps and tracks their `.log` and `.jsonl` outputs as artifacts.
- `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` expands the required 10k / 50k / 100k x TurboQuant / PqFastScan / RaBitQ matrix with load, recall, frontier, latency, and storage steps.

The suite dry-run is shape validation only. It does not execute the benchmark matrix because the current checkout does not contain the staged `data/staged-current/` TSV inputs.

## Artifacts

### `cargo-check-pg18-pgtest.log`

- Head SHA: `c143616ed2bc9ac7cffd7361018754247bff6095`
- Command:

```bash
cargo check --no-default-features --features pg18,pg_test > reviews/task-118/002-hnsw-frontier-suite-workflow/artifacts/cargo-check-pg18-pgtest.log 2>&1
```

- Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`

### `cargo-test-ecaz-cli-hnsw-frontier.log`

- Head SHA: `c143616ed2bc9ac7cffd7361018754247bff6095`
- Command:

```bash
cargo test -p ecaz-cli hnsw_frontier -- --nocapture > reviews/task-118/002-hnsw-frontier-suite-workflow/artifacts/cargo-test-ecaz-cli-hnsw-frontier.log 2>&1
```

- Key result: `2 passed; 0 failed; 0 ignored; 0 measured; 409 filtered out`

### `suite-dry-run.log`

- Head SHA: `c143616ed2bc9ac7cffd7361018754247bff6095`
- Lane: local dry-run expansion
- Fixture: `data/staged-current/ec_real_{10k,50k,100k}_{corpus,queries,manifest}.tsv/json`
- Storage formats: `turboquant`, `pq_fastscan`, `rabitq`
- Rerank mode: default HNSW exact/source rerank behavior
- Isolated surfaces: one table/index prefix per scale and storage format
- Command:

```bash
cargo run -p ecaz-cli -- bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --dry-run --manifest-output reviews/task-118/002-hnsw-frontier-suite-workflow/artifacts/suite-dry-run-manifest.json > reviews/task-118/002-hnsw-frontier-suite-workflow/artifacts/suite-dry-run.log 2>&1
```

- Key result: 45 selected suite steps expanded.

### `suite-dry-run-manifest.json`

- Head SHA: `c143616ed2bc9ac7cffd7361018754247bff6095`
- Command: produced by the `suite-dry-run.log` command above.
- Key result: 45 selected suite records with expected packet-local artifacts for load, recall, frontier, latency, and storage outputs.
