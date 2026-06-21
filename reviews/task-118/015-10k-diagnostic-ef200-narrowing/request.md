---
task: 118
packet: reviews/task-118/015-10k-diagnostic-ef200-narrowing
checkpoint_sha: 607cdecbbb5ea43c4fad497c0e7ec7d8fce61710
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: 10k Diagnostic ef=200 Narrowing

## Scope

This checkpoint narrows the Task 118 10k diagnostic-only sweeps to
`ef_search=200`.

Changed in `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`:

- 3 source-build 10k `hnsw-frontier` steps: `[40,64,100,128,160,200]` -> `[200]`
- 3 source-build 10k `hnsw-score-correlation` steps: `[40,64,100,128,160,200]` -> `[200]`
- 3 compressed-build 10k `hnsw-frontier` steps: `[40,64,100,128,160,200]` -> `[200]`
- 3 compressed-build 10k `hnsw-score-correlation` steps: `[40,64,100,128,160,200]` -> `[200]`

Recall and latency sweeps are unchanged. The final Task 118 attribution table
uses `ef_search=200`, and 50k/100k diagnostics were already narrowed to
`ef_search=200`, so this makes the 10k current-head regeneration match the
closeout evidence shape without dropping required recall/latency sweep data.

## Validation

- `jq empty crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`
  - Result: passed
- 10k frontier dry-run selected six `hnsw-frontier` steps, each with
  `--sweep 200 --queries-limit 200`.
  - Artifact: `artifacts/suite-dry-run-10k-frontier-ef200.log`
  - Artifact: `artifacts/suite-manifest-dry-run-10k-frontier-ef200.json`
- 10k score-correlation dry-run selected six `hnsw-score-correlation` steps,
  each with `--sweep 200 --queries-limit 200`.
  - Artifact: `artifacts/suite-dry-run-10k-score-correlation-ef200.log`
  - Artifact: `artifacts/suite-manifest-dry-run-10k-score-correlation-ef200.json`

No benchmark was run here. This is a suite-shape checkpoint that reduces final
diagnostic regeneration cost and keeps Task 118 focused on the closeout
decision rows.

## Remaining Task 118 Closeout Work

Regenerate current-head 10k frontier and score-correlation diagnostics with the
new `ef_search=200` shape, run Intel 50k/100k closeout suites, and update packet
006 with the final dominant-loss classification.
