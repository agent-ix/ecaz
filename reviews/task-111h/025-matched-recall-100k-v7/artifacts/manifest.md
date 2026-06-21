# Artifact Manifest

Task bucket: `reviews/task-111h/`
Packet: `reviews/task-111h/025-matched-recall-100k-v7/`
Head SHA: `008812408c648f82204732d4b826ed90803f7bb62`
Timestamp: `2026-06-20T15:05:00Z`
Branch: `bench-ivf-111g-115-attribution`

## Scope

This is a derived analysis packet. It does not run new benchmarks. It re-slices
the post-v7 100k results committed in:

- `reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-report-after-enospc.jsonl`
- `reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-rabitq4-cont-report.jsonl`
- `reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-rabitq8-cont-report.jsonl`
- `reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-turboquant-cont-report.jsonl`

Source benchmark metadata is inherited from packet 024:

- real 100k corpus,
- 200 queries,
- `k=10`,
- `nprobe` sweep `8,16,32,64,128,200`,
- rerank widths `32,64,128,256`,
- one-index-per-table prefixes in scratch database `task111h_rerank_100k_v7`,
- release PG18 build at head `bc95e5f761c96b64f4a9bf594e074888981af8fe`.

## Selection Rule

For each recall target and format, select the row with:

1. `recall@10 >= target`,
2. lowest p50 latency,
3. lowest p95 latency as tie-break,
4. lowest ec_ivf index bytes as final tie-break.

If a format cannot reach the target, report `NO_REACH` with its maximum observed
recall row in the same post-v7 100k packet.

Targets: `0.90`, `0.93`, `0.95`, `0.97`, `0.99`.

## Artifacts

- `artifacts/manifest.md`: this manifest.
- `artifacts/matched-recall-100k-v7.md`: derived matched-recall table and interpretation.

## Command

The table was generated from the packet 024 JSONL reports by joining recall,
latency, and storage rows by prefix, width, and nprobe, then applying the
selection rule above. No corpus, database, or benchmark command was run for this
analysis packet.

## Non-Claims

This packet is not a final Task 111h closeout. It covers only the 100k post-v7
warm-cache local data from packet 024. It does not replace the required full
10k/50k/100k/1M decision table, cold/remote sweep, table-owned storage decision,
legacy `0x2A` attribution, or remaining read-amplification/stage-counter work.
