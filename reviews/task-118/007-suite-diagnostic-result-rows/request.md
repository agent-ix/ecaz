# Task 118 Review Request: Suite Diagnostic Result Rows

## Scope

This checkpoint updates `ecaz bench suite` result extraction so Task 118 HNSW diagnostics appear in normalized `results.jsonl` output:

- `hnsw-frontier` logs now emit `metric=hnsw_frontier` rows.
- `hnsw-score-correlation` logs now emit `metric=hnsw_score_correlation` rows.
- The parsed rows keep the existing suite context fields such as `prefix`, `quant`, `storage_format`, `suite_database`, host, and port.

This does not change HNSW execution or benchmark behavior. It fixes the evidence pipeline for the final 50k/100k Intel pass, where candidate containment and score-correlation rows must be queryable from the suite results artifact instead of only from text logs.

## Validation

- `artifacts/cargo-test-ecaz-cli-hnsw-result-rows.log`
  - `cargo test -p ecaz-cli hnsw -- --nocapture`
  - result: `23 passed; 0 failed`

- `artifacts/suite-report-10k-diagnostic-results.log`
  - reran `bench suite report` against packet 006's existing 10k suite manifest.
  - wrote `artifacts/results-10k-with-diagnostics.jsonl`

- `artifacts/results-10k-with-diagnostics.jsonl`
  - contains `24 hnsw-frontier` rows and `24 hnsw-score-correlation` rows from the existing 10k Task 118 logs.

## Remaining Task 118 Closeout Work

The AMD host remains suitable only for relative/local checks. Final closeout still needs the Intel desktop run for full 50k and 100k source-vs-compressed HNSW evidence across TurboQuant, PqFastScan, and RaBitQ, including recall, latency, storage, frontier containment, rerank counters, and score-correlation rows.
