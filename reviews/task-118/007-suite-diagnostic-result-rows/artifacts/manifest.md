# Task 118 Packet 007 Artifact Manifest

- head SHA: `79728743811072fd007c4c310a0873c7234a919a`
- task bucket: `reviews/task-118/007-suite-diagnostic-result-rows`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: suite result parsing support for Task 118 HNSW diagnostics; reused existing 10k Task 118 artifacts from packet 006 to verify normalized extraction.
- isolated surface: existing Task 118 10k suite artifacts use one HNSW index per loaded prefix.

## Artifacts

### `cargo-test-ecaz-cli-hnsw-result-rows.log`

- command:
  `cargo test -p ecaz-cli hnsw -- --nocapture`
- purpose: focused CLI/unit coverage for HNSW loader, diagnostic commands, and suite parser rows.
- key result:
  `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 394 filtered out`

### `suite-report-10k-diagnostic-results.log`

- command:
  `cargo run -p ecaz-cli -- bench suite report --manifest reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k.json --results-output reviews/task-118/007-suite-diagnostic-result-rows/artifacts/results-10k-with-diagnostics.jsonl`
- purpose: verify that an existing Task 118 suite manifest now emits normalized diagnostic result rows from real packet-local HNSW frontier and score-correlation logs.
- key result:
  `wrote reviews/task-118/007-suite-diagnostic-result-rows/artifacts/results-10k-with-diagnostics.jsonl`

### `results-10k-with-diagnostics.jsonl`

- command: produced by the `bench suite report` command above.
- purpose: normalized JSONL proof that Task 118 final Intel runs will include diagnostic rows in `results.jsonl`, not only human-readable logs.
- line count: 170
- parsed row counts:
  - `24 hnsw-frontier hnsw_frontier`
  - `24 hnsw-score-correlation hnsw_score_correlation`
  - `24 recall recall`
  - `24 latency latency`
  - `54 storage storage_field`
  - `12 storage storage_index`
  - `8 load load_timing`
