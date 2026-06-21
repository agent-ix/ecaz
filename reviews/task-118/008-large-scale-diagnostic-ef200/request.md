# Task 118 Review Request: Large-Scale Diagnostic Sweep Narrowing

## Scope

This checkpoint narrows only the expensive large-scale Task 118 diagnostic steps:

- 50k and 100k `hnsw-frontier` steps now use `sweep: [200]`.
- 50k and 100k `hnsw-score-correlation` steps now use `sweep: [200]`.
- The change applies to both source-build and compressed-build A/B lanes for TurboQuant, PqFastScan, and RaBitQ.

Recall and latency sweeps at 50k/100k remain `[40, 64, 100, 128, 160, 200]`, so the final Intel pass still captures recall and performance curves. Storage and load coverage are unchanged.

## Rationale

The AMD-local partial run in packet 006 showed that a single 50k frontier helper could run for roughly 20 minutes on this host before interruption. The final closeout evidence belongs on the Intel desktop, but asking every large-scale diagnostic lane to run six `ef_search` values would multiply that expensive diagnostic work without adding required decision rows.

Task 118 requires candidate containment, frontier size, visited count, rerank counters, and score-correlation evidence at the required scales and formats. The decisive comparison already used in the packet is `ef_search=200`; this change preserves that row across 50k/100k and both build paths while keeping full recall/latency sweeps for curve context.

## Validation

- `jq empty crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`
  - JSON parses successfully.

- `artifacts/suite-dry-run-50k-100k-diagnostic-ef200.log`
  - `bench suite run --dry-run` for the 50k and 100k tags.
  - selected 12 frontier, 12 score-correlation, 12 recall, 12 latency, 12 load, and 12 storage steps.

- `artifacts/dry-run-diagnostic-sweep-summary.txt`
  - every selected 50k/100k frontier and score-correlation command expands with `--sweep 200 --queries-limit 200`.
  - latency commands remain full-sweep in the dry-run output.

## Remaining Task 118 Closeout Work

This is not final evidence. The Intel desktop still needs to run the 50k and 100k source-vs-compressed suite lanes and publish recall, latency, storage, frontier containment, rerank counter, and score-correlation result rows for all three formats.
