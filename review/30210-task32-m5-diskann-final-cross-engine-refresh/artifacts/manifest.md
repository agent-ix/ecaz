# Artifact Manifest

Packet: `review/30210-task32-m5-diskann-final-cross-engine-refresh`

Lane: Task 32 final post-M5 DiskANN cross-engine refresh on Apple M5.

Status: scaffold only. No measurement artifacts recorded yet.

## Intended Surface

- hardware: Apple M5
- PostgreSQL: PG18
- corpus: real10K
- isolation/shared-table surface: one-index-per-table / one-index-per-prefix
- cache state: warm cache
- release-installed extension build

## Engines

- `ec_diskann`
- `pgvectorscale`
- `ec_hnsw`

## Expected Metrics

Per engine / tuning row:

- `recall_at_10`
- `mean_ms`
- `p50_ms`
- `p95_ms`
- `p99_ms`
- `memory_hwm_kb`

Per engine:

- `build_time_s`
- `index_size_bytes`

## Normalized Output

Write `artifacts/results.jsonl` with one JSON object per engine/tuning row.

Suggested schema:

```json
{
  "family": "ec_diskann",
  "fixture": "real10k",
  "hardware": "Apple M5",
  "cache_state": "warm",
  "build_sha": "abcdef12",
  "corpus_rows": 10000,
  "dimensions": 1536,
  "tuning_label": "list_size",
  "tuning_value": 64,
  "recall_at_10": 0.9965,
  "mean_ms": 7.80,
  "p50_ms": 7.70,
  "p95_ms": 9.90,
  "p99_ms": 10.3,
  "memory_hwm_kb": 61020,
  "build_time_s": 14.59,
  "index_size_bytes": 4939776,
  "source_packet": "review/30210-task32-m5-diskann-final-cross-engine-refresh"
}
```

## Raw Artifact Checklist

- install logs
- load/build logs
- recall logs
- latency logs
- storage logs
- any compare/explain logs used for cited rows

## Commands

Populate this section with the exact benchmark commands used when the run is
executed.

## Key Result Lines

Populate this section with the exact cited result lines when the run is
executed.
