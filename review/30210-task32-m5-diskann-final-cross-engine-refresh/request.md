# Task 32 M5 DiskANN Final Cross-Engine Refresh

Reviewer: please review this benchmark packet scaffold before or after the
measurement run. This packet is intended to fill the highest-value remaining M5
benchmark gap before merging the Apple-Silicon DiskANN work.

## Objective

Refresh the Task 29d cross-engine comparison on the final M5 DiskANN code
state so the public benchmark inventory can cite a packet-backed post-M5 row
instead of only the pre-round baseline.

This is a benchmark/reporting packet, not a new optimization slice.

Suite config:

- `review/30210-task32-m5-diskann-final-cross-engine-refresh/task32-m5-diskann-final-cross-engine.packet.json`

## Benchmark Surface

- hardware: Apple M5 local development machine
- PostgreSQL: PG18
- corpus: real10K
- isolation: one-index-per-table / one-index-per-prefix
- cache state: warm cache, to match the original Task 29d comparison surface
- release-installed extension build

## Engines

1. `ec_diskann`
   - final M5 branch code state
   - `graph_degree=32`
   - `build_list_size=100`
   - `alpha=1.2`
   - search-list sweep: `64,128,200,400,800`
2. `pgvectorscale`
   - same comparison shape as Task 29d
3. `ec_hnsw`
   - same comparison shape as Task 29d

## Metrics To Capture

Per engine / tuning point:

- recall@10
- mean query time if emitted by the benchmark surface
- latency p50 / p95 / p99
- memory HWM

Per engine:

- build time
- index size

## Required Artifacts

Store all raw logs in `artifacts/` and keep `artifacts/manifest.md` as the
packet-local source of truth.

Add a normalized `artifacts/results.jsonl` with one object per engine/tuning
row using the schema recorded in `artifacts/manifest.md`.

Suggested suite flow:

1. `ecaz bench suite audit --config <packet-json>`
2. `ecaz bench suite run --config <packet-json> --manifest-output ... --results-output ...`
3. `ecaz bench suite report --manifest <suite-manifest.json> --results-output ...`

Expected raw artifact categories:

- install logs
- corpus load logs
- recall logs
- latency logs
- storage logs
- any compare/explain logs used for cited rows

## Output Goal

This packet should provide enough normalized output to update:

- `docs/benchmarks.md`
- `docs/benchmark-index.md`

without re-reading all raw logs.

## Validation

This packet makes measurement claims. Record the exact commands used and note
whether any tests were run alongside the benchmark refresh. If tests are
skipped, say so explicitly.
